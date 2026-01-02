use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, LitStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Endian {
    Big,
    Little,
}

impl Endian {
    fn wrapper_path_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Endian::Big => quote!(::simple_endian::BigEndian),
            Endian::Little => quote!(::simple_endian::LittleEndian),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextEncoding {
    Utf8,
    Utf16,
    Utf32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextPad {
    Null,
    Space,
}

fn parse_container_endian(attrs: &[Attribute]) -> Result<Endian, Error> {
    for attr in attrs {
        if !attr.path().is_ident("endian") {
            continue;
        }

        // Accept: #[endian(be)] or #[endian(le)]
        let ident = attr.parse_args::<syn::Ident>()?;
        let s = ident.to_string();
        return match s.as_str() {
            "be" | "big" | "big_endian" => Ok(Endian::Big),
            "le" | "little" | "little_endian" => Ok(Endian::Little),
            _ => Err(Error::new(
                ident.span(),
                "invalid endian; expected `be` or `le`",
            )),
        };
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        "missing #[endian(be)] or #[endian(le)] on type deriving Endianize",
    ))
}

fn parse_enum_repr_int(attrs: &[Attribute]) -> Result<syn::Ident, Error> {
    // Require one of: #[repr(u8)], #[repr(u16)], #[repr(u32)], #[repr(u64)]
    // (We intentionally don't accept isize/usize or C here for a stable, explicit on-wire tag.)
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }

        // Parse the first repr ident.
        // We keep this simple: take the first ident and validate it.
        let ident = attr.parse_args::<syn::Ident>()?;
        let s = ident.to_string();
        match s.as_str() {
            "u8" | "u16" | "u32" | "u64" => return Ok(ident),
            _ => {
                return Err(Error::new(
                    ident.span(),
                    "Endianize enums require an explicit #[repr(u8|u16|u32|u64)]",
                ))
            }
        }
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        "Endianize enums require an explicit #[repr(u8|u16|u32|u64)]",
    ))
}

fn has_text_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("text"))
}

fn is_fixed_text_wire_type(ty: &syn::Type) -> bool {
    // Heuristic: if a user explicitly uses one of our fixed UTF wire leaf types
    // (which already incorporate endian via their internal code units), we
    // should NOT wrap it in BigEndian/LittleEndian.
    //
    // This keeps `#[derive(Endianize)]` usable for structs that want to spell
    // the field type directly instead of using `#[text(...)]`.
    let syn::Type::Path(p) = ty else { return false };
    let Some(seg) = p.path.segments.last() else { return false };
    matches!(
        seg.ident.to_string().as_str(),
        "FixedUtf8NullPadded"
            | "FixedUtf8SpacePadded"
            |
        "FixedUtf16BeNullPadded"
            | "FixedUtf16BeSpacePadded"
            | "FixedUtf16LeNullPadded"
            | "FixedUtf16LeSpacePadded"
            | "FixedUtf32BeNullPadded"
            | "FixedUtf32BeSpacePadded"
            | "FixedUtf32LeNullPadded"
            | "FixedUtf32LeSpacePadded"
    )
}

fn is_u8_array_type(ty: &syn::Type) -> bool {
    let syn::Type::Array(arr) = ty else {
        return false;
    };

    match &*arr.elem {
        syn::Type::Path(p) => p.path.is_ident("u8"),
        _ => false,
    }
}

fn array_elem_and_len(ty: &syn::Type) -> Option<(&syn::Type, &syn::Expr)> {
    let syn::Type::Array(arr) = ty else {
        return None;
    };
    Some((&*arr.elem, &arr.len))
}

fn parse_text_attr(attrs: &[Attribute]) -> Result<(TextEncoding, usize, TextPad), Error> {
    // Supported:
    //   #[text(utf16, units = 16, pad = "space")]
    //   #[text(utf32, units = 8,  pad = "null")]

    let attr = attrs
        .iter()
        .find(|a| a.path().is_ident("text"))
        .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "missing #[text(...)]"))?;

    let mut encoding: Option<TextEncoding> = None;
    let mut units: Option<usize> = None;
    let mut pad: Option<TextPad> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("utf8") {
            encoding = Some(TextEncoding::Utf8);
            return Ok(());
        }
        if meta.path.is_ident("utf16") {
            encoding = Some(TextEncoding::Utf16);
            return Ok(());
        }
        if meta.path.is_ident("utf32") {
            encoding = Some(TextEncoding::Utf32);
            return Ok(());
        }

        if meta.path.is_ident("units") {
            let lit: syn::LitInt = meta.value()?.parse()?;
            units = Some(lit.base10_parse()?);
            return Ok(());
        }

        if meta.path.is_ident("pad") {
            let lit: LitStr = meta.value()?.parse()?;
            let s = lit.value();
            pad = Some(match s.as_str() {
                "null" => TextPad::Null,
                "space" => TextPad::Space,
                _ => {
                    return Err(Error::new(
                        lit.span(),
                        "invalid pad; expected \"null\" or \"space\"",
                    ))
                }
            });
            return Ok(());
        }

        Err(Error::new(
            meta.path.span(),
            "unknown text option; expected utf8/utf16/utf32, units = N, pad = \"null\"|\"space\"",
        ))
    })?;

    let encoding = encoding.ok_or_else(|| {
        Error::new(attr.span(), "text encoding missing; expected utf8, utf16, or utf32")
    })?;
    let units = units.ok_or_else(|| Error::new(attr.span(), "text units missing; expected units = N"))?;
    let pad = pad.unwrap_or(TextPad::Null);

    Ok((encoding, units, pad))
}

pub fn derive_endianize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_endianize_inner(&input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_endianize_inner(input: &DeriveInput) -> Result<TokenStream, Error> {
    let endian = parse_container_endian(&input.attrs)?;
    let wrapper_path = endian.wrapper_path_tokens();

    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let wire_name = format_ident!("{}Wire", name);

    let mut wire_field_idents: Vec<syn::Ident> = Vec::new();
    let mut logical_field_idents: Vec<syn::Ident> = Vec::new();
    let mut logical_field_types: Vec<syn::Type> = Vec::new();
    let mut logical_is_text: Vec<bool> = Vec::new();
    let mut is_union = false;
    let wire_item = match &input.data {
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => {
                    let mut wire_fields = Vec::with_capacity(fields.named.len());

                    for f in &fields.named {
                        let f_ident = f
                            .ident
                            .as_ref()
                            .ok_or_else(|| Error::new(f.span(), "expected named field"))?;

                        wire_field_idents.push(f_ident.clone());
                        logical_field_idents.push(f_ident.clone());
                        logical_field_types.push(f.ty.clone());
                        logical_is_text.push(has_text_attr(&f.attrs));

                        let ty = &f.ty;

                        // If this field has #[text(...)] we force its wire type.
                        let wire_ty = if has_text_attr(&f.attrs) {
                            let (enc, units, pad) = parse_text_attr(&f.attrs)?;
                            let units_lit = syn::LitInt::new(&units.to_string(), f.span());
                            match (enc, pad, endian) {
                                (TextEncoding::Utf8, TextPad::Null, _) => {
                                    quote!(::simple_endian::FixedUtf8NullPadded<#units_lit>)
                                }
                                (TextEncoding::Utf8, TextPad::Space, _) => {
                                    quote!(::simple_endian::FixedUtf8SpacePadded<#units_lit>)
                                }
                                (TextEncoding::Utf16, TextPad::Null, Endian::Big) => {
                                    quote!(::simple_endian::FixedUtf16BeNullPadded<#units_lit>)
                                }
                                (TextEncoding::Utf16, TextPad::Space, Endian::Big) => {
                                    quote!(::simple_endian::FixedUtf16BeSpacePadded<#units_lit>)
                                }
                                (TextEncoding::Utf16, TextPad::Null, Endian::Little) => {
                                    quote!(::simple_endian::FixedUtf16LeNullPadded<#units_lit>)
                                }
                                (TextEncoding::Utf16, TextPad::Space, Endian::Little) => {
                                    quote!(::simple_endian::FixedUtf16LeSpacePadded<#units_lit>)
                                }
                                (TextEncoding::Utf32, TextPad::Null, Endian::Big) => {
                                    quote!(::simple_endian::FixedUtf32BeNullPadded<#units_lit>)
                                }
                                (TextEncoding::Utf32, TextPad::Space, Endian::Big) => {
                                    quote!(::simple_endian::FixedUtf32BeSpacePadded<#units_lit>)
                                }
                                (TextEncoding::Utf32, TextPad::Null, Endian::Little) => {
                                    quote!(::simple_endian::FixedUtf32LeNullPadded<#units_lit>)
                                }
                                (TextEncoding::Utf32, TextPad::Space, Endian::Little) => {
                                    quote!(::simple_endian::FixedUtf32LeSpacePadded<#units_lit>)
                                }
                            }
                        } else if is_fixed_text_wire_type(ty) {
                            quote!(#ty)
                        } else if is_u8_array_type(ty) {
                            // Raw bytes are already wire-safe; endianness doesn't apply.
                            quote!(#ty)
                        } else if let Some((elem_ty, len_expr)) = array_elem_and_len(ty) {
                            // For fixed-size arrays, apply the container endian to each element.
                            // Example: `[u16; 8]` -> `[LittleEndian<u16>; 8]` (when #[endian(le)]).
                            quote!([#wrapper_path<#elem_ty>; #len_expr])
                        } else {
                            // Default: wrap the user-specified field type in the container endian.
                            quote!(#wrapper_path<#ty>)
                        };

                        wire_fields.push(quote!(pub #f_ident: #wire_ty));
                    }

                    quote!({
                        #(#wire_fields,)*
                    })
                }
                Fields::Unnamed(fields) => {
                    let mut wire_fields = Vec::with_capacity(fields.unnamed.len());
                    for f in &fields.unnamed {
                        if has_text_attr(&f.attrs) {
                            return Err(Error::new(
                                f.span(),
                                "#[text(...)] is only supported on named fields for now",
                            ));
                        }
                        let ty = &f.ty;
                        wire_fields.push(quote!(#wrapper_path<#ty>));
                    }
                    quote!((#(#wire_fields,)*))
                }
                Fields::Unit => quote!(;),
            };

            let wire = quote! {
                #[repr(C)]
                #[allow(non_camel_case_types)]
                #vis struct #wire_name #generics #fields
            };

            wire
        }
        Data::Enum(data) => {
            // Enum support: generate `EnumWire` as a tag + payload union.
            // Restrictions for v1:
            // - enum must have #[repr(u8|u16|u32|u64)]
            // - supported variants: unit variants and *named-field* variants
            // - tuple variants are rejected for now
            let tag_int = parse_enum_repr_int(&input.attrs)?;
            let tag_ty = quote!(#wrapper_path<#tag_int>);

            let payload_name = format_ident!("{}WirePayload", name);

            let mut any_payload = false;
            let mut payload_structs = Vec::<proc_macro2::TokenStream>::new();
            let mut payload_union_fields = Vec::<proc_macro2::TokenStream>::new();
            let mut variant_arms_read = Vec::<proc_macro2::TokenStream>::new();
            let mut variant_arms_write = Vec::<proc_macro2::TokenStream>::new();

            for v in &data.variants {
                let v_ident = &v.ident;
                let v_payload_struct = format_ident!("{}WirePayload_{}", name, v_ident);
                let v_payload_union_field = format_ident!("{}", v_ident);
                let v_tag_const = format_ident!("__{}_TAG_{}", name, v_ident);

                match &v.fields {
                    Fields::Unit => {
                        // Unit variants: no payload.
                        let disc_expr = v
                            .discriminant
                            .as_ref()
                            .ok_or_else(|| {
                                Error::new(
                                    v.span(),
                                    "Endianize enums require explicit discriminants for all variants, e.g. `Variant = 1`",
                                )
                            })?
                            .1
                            .clone();
                        payload_structs.push(quote! {
                            #[allow(non_upper_case_globals)]
                            const #v_tag_const: #tag_int = (#disc_expr) as #tag_int;
                        });
                        variant_arms_read.push(quote! {
                            x if x == #v_tag_const => {
                                Ok(#wire_name { tag: #v_tag_const.into(), payload: #payload_name { _unused: [] } })
                            }
                        });
                        variant_arms_write.push(quote! {
                            x if x == #v_tag_const => {
                                Ok(())
                            }
                        });
                    }
                    Fields::Named(fields) => {
                        any_payload = true;

                        // Require an explicit discriminant for data-carrying variants.
                        // Rust doesn't allow casting such variants to integers.
                        let disc_expr = v
                            .discriminant
                            .as_ref()
                            .ok_or_else(|| {
                                Error::new(
                                    v.span(),
                                    "Endianize enums with payload require explicit discriminants, e.g. `Variant = 1`",
                                )
                            })?
                            .1
                            .clone();

                        payload_structs.push(quote! {
                            #[allow(non_upper_case_globals)]
                            const #v_tag_const: #tag_int = (#disc_expr) as #tag_int;
                        });

                        let mut field_idents = Vec::with_capacity(fields.named.len());
                        let mut field_defs = Vec::with_capacity(fields.named.len());
                        let mut reads = Vec::with_capacity(fields.named.len());
                        let mut writes = Vec::with_capacity(fields.named.len());

                        for f in &fields.named {
                            let f_ident = f
                                .ident
                                .as_ref()
                                .ok_or_else(|| Error::new(f.span(), "expected named field"))?;
                            field_idents.push(f_ident);
                            let ty = &f.ty;

                            let wire_ty = if has_text_attr(&f.attrs) {
                                let (enc, units, pad) = parse_text_attr(&f.attrs)?;
                                let units_lit = syn::LitInt::new(&units.to_string(), f.span());
                                match (enc, pad, endian) {
                                    (TextEncoding::Utf8, TextPad::Null, _) => {
                                        quote!(::simple_endian::FixedUtf8NullPadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf8, TextPad::Space, _) => {
                                        quote!(::simple_endian::FixedUtf8SpacePadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf16, TextPad::Null, Endian::Big) => {
                                        quote!(::simple_endian::FixedUtf16BeNullPadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf16, TextPad::Space, Endian::Big) => {
                                        quote!(::simple_endian::FixedUtf16BeSpacePadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf16, TextPad::Null, Endian::Little) => {
                                        quote!(::simple_endian::FixedUtf16LeNullPadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf16, TextPad::Space, Endian::Little) => {
                                        quote!(::simple_endian::FixedUtf16LeSpacePadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf32, TextPad::Null, Endian::Big) => {
                                        quote!(::simple_endian::FixedUtf32BeNullPadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf32, TextPad::Space, Endian::Big) => {
                                        quote!(::simple_endian::FixedUtf32BeSpacePadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf32, TextPad::Null, Endian::Little) => {
                                        quote!(::simple_endian::FixedUtf32LeNullPadded<#units_lit>)
                                    }
                                    (TextEncoding::Utf32, TextPad::Space, Endian::Little) => {
                                        quote!(::simple_endian::FixedUtf32LeSpacePadded<#units_lit>)
                                    }
                                }
                            } else if is_fixed_text_wire_type(ty) {
                                quote!(#ty)
                            } else if is_u8_array_type(ty) {
                                // Raw bytes are already wire-safe; endianness doesn't apply.
                                quote!(#ty)
                            } else if let Some((elem_ty, len_expr)) = array_elem_and_len(ty) {
                                // For fixed-size arrays, apply the container endian to each element.
                                quote!([#wrapper_path<#elem_ty>; #len_expr])
                            } else {
                                quote!(#wrapper_path<#ty>)
                            };

                            field_defs.push(quote!(pub #f_ident: #wire_ty));
                            reads.push(quote!(#f_ident: ::simple_endian::read_specific(reader)?));
                            writes.push(quote!(::simple_endian::write_specific(writer, &payload.#f_ident)?;));
                        }

                        payload_structs.push(quote! {
                            #[repr(C)]
                            #[allow(non_camel_case_types)]
                            #vis struct #v_payload_struct #generics {
                                #(#field_defs,)*
                            }
                        });

                        payload_union_fields.push(quote!(#v_payload_union_field: ::std::mem::ManuallyDrop<#v_payload_struct #ty_generics>));

                        // Read arm: read payload struct, store in union.
                        variant_arms_read.push(quote! {
                            x if x == #v_tag_const => {
                                let payload = #v_payload_struct { #(#reads,)* };
                                Ok(#wire_name {
                                    tag: #v_tag_const.into(),
                                    payload: #payload_name { #v_payload_union_field: ::std::mem::ManuallyDrop::new(payload) },
                                })
                            }
                        });

                        // Write arm: reinterpret union as the variant payload and write fields.
                        variant_arms_write.push(quote! {
                            x if x == #v_tag_const => {
                                // SAFETY: The active union field is selected by the tag.
                                let payload = unsafe { &*self.payload.#v_payload_union_field };
                                #(#writes)*
                                Ok(())
                            }
                        });
                    }
                    Fields::Unnamed(_) => {
                        return Err(Error::new(
                            v.span(),
                            "Endianize enums: tuple variants are not supported yet; use named fields",
                        ));
                    }
                }
            }

            // Payload union: if there are no payload variants, use a zero-sized placeholder.
            let payload_def = if any_payload {
                quote! {
                    #[repr(C)]
                    #[allow(non_snake_case)]
                    #vis union #payload_name #generics {
                        #(#payload_union_fields,)*
                        // Ensure the union is not empty.
                        _unused: [u8; 0],
                    }
                }
            } else {
                quote! {
                    #[repr(C)]
                    #vis union #payload_name #generics {
                        _unused: [u8; 0],
                    }
                }
            };

            let wire = quote! {
                #(#payload_structs)*

                #payload_def

                #[repr(C)]
                #[allow(non_camel_case_types)]
                #vis struct #wire_name #generics {
                    pub tag: #tag_ty,
                    pub payload: #payload_name #ty_generics,
                }

                #[cfg(feature = "io-std")]
                impl #impl_generics ::simple_endian::EndianRead for #wire_name #ty_generics #where_clause {
                    fn read_from<R: ::std::io::Read>(reader: &mut R) -> ::std::io::Result<Self> {
                        let tag: #tag_ty = ::simple_endian::read_specific(reader)?;
                        let raw: #tag_int = tag.into();
                        match raw {
                            #(#variant_arms_read,)*
                            _ => Err(::std::io::Error::new(
                                ::std::io::ErrorKind::InvalidData,
                                format!("invalid {} tag: {}", stringify!(#name), raw),
                            )),
                        }
                    }
                }

                #[cfg(feature = "io-std")]
                impl #impl_generics ::simple_endian::EndianWrite for #wire_name #ty_generics #where_clause {
                    fn write_to<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                        ::simple_endian::write_specific(writer, &self.tag)?;
                        let raw: #tag_int = self.tag.into();
                        match raw {
                            #(#variant_arms_write,)*
                            _ => Err(::std::io::Error::new(
                                ::std::io::ErrorKind::InvalidData,
                                "invalid enum tag for payload",
                            )),
                        }
                    }
                }
            };

            wire
        }
        Data::Union(data) => {
            is_union = true;

            // Union support (safe default): generate `UnionWire` but DO NOT generate IO impls.
            // Like structs, each field type is wrapped with the container endian wrapper.
            // We currently do not support #[text(...)] on union fields.

            let mut wire_fields = Vec::with_capacity(data.fields.named.len());
            for f in &data.fields.named {
                let f_ident = f
                    .ident
                    .as_ref()
                    .ok_or_else(|| Error::new(f.span(), "expected named union field"))?;

                if has_text_attr(&f.attrs) {
                    return Err(Error::new(
                        f.span(),
                        "#[text(...)] is not supported on union fields",
                    ));
                }

                let ty = &f.ty;
                // Unions require Copy or ManuallyDrop at the union-level; we don't enforce here.
                // Users can use `ManuallyDrop<T>` in their union fields if needed.
                wire_fields.push(quote!(#f_ident: #wrapper_path<#ty>));
            }

            quote! {
                #[repr(C)]
                #[allow(non_camel_case_types)]
                #vis union #wire_name #generics {
                    #(#wire_fields,)*
                }
            }
        }
    };

    // If we have named fields, we can generate IO impls by reading/writing each field in order.
    // (Tuple structs can be added later; named fields cover the main repr(C) wire-layout use-case.)
    let io_impls = if !wire_field_idents.is_empty() && !is_union {
        let reads = wire_field_idents.iter().map(|f| {
            quote!(#f: ::simple_endian::read_specific(reader)?)
        });
        let writes = wire_field_idents.iter().map(|f| {
            quote!(::simple_endian::write_specific(writer, &self.#f)?;)
        });

        quote! {
            #[cfg(feature = "io-std")]
            impl #impl_generics ::simple_endian::EndianRead for #wire_name #ty_generics #where_clause {
                fn read_from<R: ::std::io::Read>(reader: &mut R) -> ::std::io::Result<Self> {
                    Ok(Self { #(#reads,)* })
                }
            }

            #[cfg(feature = "io-std")]
            impl #impl_generics ::simple_endian::EndianWrite for #wire_name #ty_generics #where_clause {
                fn write_to<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    #(#writes)*
                    Ok(())
                }
            }
        }
    } else {
        // Unit / tuple structs: no IO impls for now.
        quote! {}
    };

    // Struct conversions:
    // - `From<Logical> for Wire` is always infallible for named-field structs because:
    //   * endian wrappers accept `T: Into<Wrapper<T>>` via `.into()`
    //   * fixed text wire fields support `TryFrom<&str>` / `TryFrom<String>`; the logical source is a `String`
    //     but we convert by borrowing `&str` (may fail if it doesn't fit), so we keep this direction infallible
    //     ONLY when there are no #[text] fields.
    // - `TryFrom<Wire> for Logical` can fail for text fields (invalid encoding), so we model that explicitly.
    let has_any_text = logical_is_text.iter().any(|&b| b);
    let struct_conversions = if !wire_field_idents.is_empty() && !is_union {
        // From<Logical> for Wire: only generate if there are no #[text] fields.
        let from_logical_for_wire = if !has_any_text {
            let assigns = logical_field_idents
                .iter()
                .zip(logical_field_types.iter())
                .map(|(f, ty)| {
                    if is_u8_array_type(ty) {
                        quote!(#f: v.#f)
                    } else if array_elem_and_len(ty).is_some() {
                        quote!(#f: v.#f.map(::core::convert::Into::into))
                    } else {
                        quote!(#f: v.#f.into())
                    }
                });
            quote! {
                impl #impl_generics ::core::convert::From<#name #ty_generics> for #wire_name #ty_generics #where_clause {
                    fn from(v: #name #ty_generics) -> Self {
                        Self { #(#assigns,)* }
                    }
                }
            }
        } else {
            quote! {}
        };

        // TryFrom<Wire> for Logical: always generate for structs with named fields.
        // Numeric fields: `.to_native()`
        // Text fields: `String::try_from(&wire_field)`
        let try_assigns = logical_field_idents
            .iter()
            .zip(logical_field_types.iter())
            .zip(logical_is_text.iter())
            .map(|((f, ty), is_text)| {
                if *is_text {
                    quote!(#f: ::std::string::String::try_from(&v.#f).map_err(|e| ::simple_endian::FixedTextError::from(e))?)
                } else if is_u8_array_type(ty) {
                    quote!(#f: v.#f)
                } else if array_elem_and_len(ty).is_some() {
                    quote!(#f: v.#f.map(|x| x.to_native()))
                } else {
                    quote!(#f: v.#f.to_native())
                }
            });

        // Choose error type:
        // `String::try_from(&FixedText)` uses `simple_endian::FixedTextError`.
        // This impl also requires `alloc` (for `String`) and `text_fixed`.
        let try_from_wire_for_logical = if has_any_text {
            quote! {
                #[cfg(all(feature = "simple_string_impls", feature = "text_fixed"))]
                impl #impl_generics ::core::convert::TryFrom<#wire_name #ty_generics> for #name #ty_generics #where_clause {
                    type Error = ::simple_endian::FixedTextError;

                    fn try_from(v: #wire_name #ty_generics) -> Result<Self, Self::Error> {
                        Ok(Self { #(#try_assigns,)* })
                    }
                }
            }
        } else {
            quote! {
                impl #impl_generics ::core::convert::From<#wire_name #ty_generics> for #name #ty_generics #where_clause {
                    fn from(v: #wire_name #ty_generics) -> Self {
                        Self { #(#try_assigns,)* }
                    }
                }
            }
        };

        quote! {
            #from_logical_for_wire
            #try_from_wire_for_logical
        }
    } else {
        quote! {}
    };

    // Note: For now we just generate the wire type + aliases. Conversions can be added next.
    let expanded = quote! {
        #wire_item

        #io_impls

        #struct_conversions

        // Preserve where-clause usage for future impls.
        const _: () = {
            fn _assert_where_clause #impl_generics () #where_clause {}
        };
    };

    Ok(expanded.into())
}
