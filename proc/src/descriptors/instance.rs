use {
    super::{
        buffer,
        layout::layout_type_name,
        parse::{DescriptorType, Input},
    },
    proc_macro2::TokenStream,
};

pub(super) fn instance_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Instance", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);
    let elem_ident = quote::format_ident!("{}Elem", instance_ident);

    let descriptors: TokenStream = input
        .descriptors
        .iter()
        .filter_map(|input| match &input.ty {
            DescriptorType::Sampler(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::Sampler>,
                ))
            }
            DescriptorType::SampledImage(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::ImageViewDescriptor>,
                ))
            }
            DescriptorType::CombinedImageSampler(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::CombinedImageSampler>,
                ))
            }
            DescriptorType::Buffer(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::BufferRange>,
                ))
            }
            DescriptorType::AccelerationStructure(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::AccelerationStructure>,
                ))
            }
        })
        .collect();

    let update_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .filter_map(|input| {
            let field = &input.member;

            let descriptor_field =
                quote::format_ident!("descriptor_{}", input.member);
            let write_descriptor =
                quote::format_ident!("write_{}_descriptor", input.member);

            let stream = match &input.ty {
                DescriptorType::Sampler(_attr) => {
                    quote::quote!(
                        let #write_descriptor;
                        match &elem.#descriptor_field {
                            Some(sampler) => {
                                if input.#field == *sampler {
                                    #write_descriptor = false;
                                } else {
                                    elem.#descriptor_field = Some(std::clone::Clone::clone(&input.#field));
                                    #write_descriptor = true;
                                }
                            }
                            _ => {
                                elem.#descriptor_field = Some(std::clone::Clone::clone(&input.#field));
                                #write_descriptor = true;
                            }
                        }
                    )
                }
                DescriptorType::SampledImage(_attr) => {
                    quote::quote!(
                        let #write_descriptor;
                        match &elem.#descriptor_field {
                            Some(::sierra::ImageViewDescriptor { view, layout: ::sierra::Layout::ShaderReadOnlyOptimal }) => {
                                if ::sierra::SampledImage::eq(&input.#field, view) {
                                    #write_descriptor = false;
                                } else {
                                    let view = ::sierra::SampledImage::get_view(&input.#field, device)?;
                                    elem.#descriptor_field = Some(::sierra::ImageViewDescriptor {
                                        view,
                                        layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                    });
                                    #write_descriptor = true;
                                }
                            }
                            _ => {
                                let view = ::sierra::SampledImage::get_view(&input.#field, device)?;
                                elem.#descriptor_field = Some(::sierra::ImageViewDescriptor {
                                    view,
                                    layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                });
                                #write_descriptor = true;
                            }
                        }
                    )
                }
                DescriptorType::CombinedImageSampler(attr) => {
                    let sampler = &attr.sampler;
                    quote::quote!(
                        let #write_descriptor;
                        match &elem.#descriptor_field {
                            Some(::sierra::CombinedImageSampler { view, sampler, layout: ::sierra::Layout::ShaderReadOnlyOptimal }) => {
                                if ::sierra::SampledImage::eq(&input.#field, view) && input.#sampler == *sampler {
                                    #write_descriptor = false;
                                } else {
                                    let view = ::sierra::SampledImage::get_view(&input.#field, device)?;
                                    elem.#descriptor_field = Some(::sierra::CombinedImageSampler {
                                        view,
                                        sampler: std::clone::Clone::clone(&input.#sampler),
                                        layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                    });
                                    #write_descriptor = true;
                                }
                            }
                            _ => {
                                let view = ::sierra::SampledImage::get_view(&input.#field, device)?;
                                elem.#descriptor_field = Some(::sierra::CombinedImageSampler {
                                    view,
                                    sampler: std::clone::Clone::clone(&input.#sampler),
                                    layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                });
                                #write_descriptor = true;
                            }
                        }
                    )
                }
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Uniform,
                    ..
                }) => quote::quote!(
                    let #write_descriptor;
                    match &elem.#descriptor_field {
                        Some(range) => {
                            if ::sierra::UniformBuffer::eq(&input.#field, range) {
                                #write_descriptor = false;
                            } else {
                                let range = ::sierra::UniformBuffer::get_range(&input.#field, device)?;
                                elem.#descriptor_field = Some(range);
                                #write_descriptor = true;
                            }
                        }
                        _ => {
                            let range = ::sierra::UniformBuffer::get_range(&input.#field, device)?;
                            elem.#descriptor_field = Some(range);
                            #write_descriptor = true;
                        }
                    }
                ),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Storage,
                    ..
                }) => quote::quote!(
                    let #write_descriptor;
                    match &elem.#descriptor_field {
                        Some(range) => {
                            if ::sierra::StorageBuffer::eq(&input.#field, range) {
                                #write_descriptor = false;
                            } else {
                                let range = ::sierra::StorageBuffer::get_range(&input.#field, device)?;
                                elem.#descriptor_field = Some(range);
                                #write_descriptor = true;
                            }
                        }
                        _ => {
                            let range = ::sierra::StorageBuffer::get_range(&input.#field, device)?;
                            elem.#descriptor_field = Some(range);
                            #write_descriptor = true;
                        }
                    }
                ),
                DescriptorType::AccelerationStructure(_) => quote::quote!(
                    let #write_descriptor;
                    match &elem.#descriptor_field {
                        Some(accel) => {
                            if accel == &input.#field {
                                #write_descriptor = false;
                            } else {
                                elem.#descriptor_field = Some(::std::clone::Clone::clone(&input.#field));
                                #write_descriptor = true;
                            }
                        }
                        _ => {
                            elem.#descriptor_field = Some(::std::clone::Clone::clone(&input.#field));
                            #write_descriptor = true;
                        }
                    }
                ),
            };

            Some(stream)
        })
        .collect();

    let mut binding = 0u32;
    let write_updated_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .filter_map(|input| {
            let descriptors = match input.ty {
                DescriptorType::Sampler(_) => Some(quote::quote!(::sierra::Descriptors::Sampler(
                    std::slice::from_ref(descriptor)
                ))),
                DescriptorType::SampledImage(_) => Some(quote::quote!(
                    ::sierra::Descriptors::SampledImage(std::slice::from_ref(descriptor))
                )),
                DescriptorType::CombinedImageSampler(_) => Some(quote::quote!(
                    ::sierra::Descriptors::CombinedImageSampler(std::slice::from_ref(descriptor))
                )),
                DescriptorType::AccelerationStructure(_) => Some(quote::quote!(
                    ::sierra::Descriptors::AccelerationStructure(std::slice::from_ref(descriptor))
                )),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Uniform,
                    ..
                }) => Some(quote::quote!(::sierra::Descriptors::UniformBuffer(
                    std::slice::from_ref(descriptor)
                ))),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Storage,
                    ..
                }) => Some(quote::quote!(::sierra::Descriptors::StorageBuffer(
                    std::slice::from_ref(descriptor)
                ))),
            }?;

            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            let write_descriptor = quote::format_ident!("write_{}_descriptor", input.member);

            let stream = quote::quote!(
                if #write_descriptor {
                    let descriptor: &_ = elem.#descriptor_field.as_ref().unwrap();
                    writes.extend(Some(::sierra::WriteDescriptorSet {
                        set: &elem.set,
                        binding: #binding,
                        element: 0,
                        descriptors: #descriptors,
                    }));
                }
            );

            binding += 1;
            Some(stream)
        })
        .collect();

    let updated_descriptor_assertions: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            quote::quote!(
                debug_assert!(elem.#descriptor_field.is_some());
            )
        })
        .collect();

    let new_cycle_elem_descriptors: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            quote::quote!(
                #descriptor_field: ::std::option::Option::None,
            )
        })
        .collect();

    let vis = &input.item_struct.vis;
    let uniforms_ident = quote::format_ident!("{}Uniforms", input.item_struct.ident);

    let uniforms_field = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(pub uniforms_buffer: ::std::option::Option<(#uniforms_ident, ::sierra::BufferRange)>,)
    };

    let new_cycle_elem_uniforms_buffer = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(uniforms_buffer: ::std::option::Option::None,)
    };

    let update_uniforms_statement = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            let write_uniforms;
            if elem.uniforms_buffer.is_none() {
                let mut uniforms: #uniforms_ident = ::sierra::Zeroable::zeroed();
                uniforms.copy_from_input(input);
                let buffer = device.create_buffer(::sierra::BufferInfo {
                    align: 255,
                    size: ::std::convert::TryFrom::try_from(::std::mem::size_of::<#uniforms_ident>() as u64).map_err(|_| ::sierra::OutOfMemory)?,
                    usage: ::sierra::BufferUsage::UNIFORM | ::sierra::BufferUsage::TRANSFER_DST,
                })?;

                elem.uniforms_buffer = Some((uniforms, buffer.into()));
                write_uniforms = true;
            } else {
                write_uniforms = false;
                elem.uniforms_buffer.as_mut().unwrap().0.copy_from_input(input);
            }
        )
    };

    let write_uniforms_statement = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            if write_uniforms {
                writes.extend(Some(::sierra::WriteDescriptorSet {
                    set: &elem.set,
                    binding: #binding,
                    element: 0,
                    descriptors: ::sierra::Descriptors::UniformBuffer(::std::slice::from_ref(&elem.uniforms_buffer.as_ref().unwrap().1)),
                }));
            }

            let (uniforms, buffer) = elem.uniforms_buffer.as_ref().unwrap();
            encoder.update_buffer(&buffer.buffer, 0, ::std::slice::from_ref(uniforms));
        )
    };

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"[`sierra::DescriptorsInstance`] implementation for [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    quote::quote!(
        #doc_attr
        #vis struct #instance_ident {
            pub layout: ::sierra::DescriptorSetLayout,
            pub cycle: ::std::vec::Vec<#elem_ident>,
        }

        #doc_attr
        #vis struct #elem_ident {
            pub set: ::sierra::DescriptorSet,
            #descriptors
            #uniforms_field
        }

        impl ::sierra::UpdatedDescriptors for #elem_ident {
            fn raw(&self) -> &::sierra::DescriptorSet{
                &self.set
            }
        }

        impl #instance_ident {
            pub fn new(layout: &#layout_ident) -> Self {
                #instance_ident {
                    layout: layout.layout.clone(),
                    cycle: ::std::vec::Vec::new(),
                }
            }

            pub fn update<'a>(
                &'a mut self,
                input: &#ident,
                fence: usize,
                device: &::sierra::Device,
                writes: &mut impl ::std::iter::Extend<::sierra::WriteDescriptorSet<'a>>,
                encoder: &mut ::sierra::Encoder<'a>,
            ) -> ::std::result::Result<&'a #elem_ident, ::sierra::OutOfMemory> {
                while self.cycle.len() <= fence {
                        let new_elem = self.new_cycle_elem(device)?;
                        self.cycle.push(new_elem);
                }

                let elem = self.cycle.get_mut(fence).unwrap();
                #update_uniforms_statement
                #update_descriptor_statements

                let elem = self.cycle.get(fence).unwrap();
                #write_uniforms_statement
                #write_updated_descriptor_statements

                #updated_descriptor_assertions

                ::std::result::Result::Ok(elem)
            }

            pub fn raw_layout(&self) -> &::sierra::DescriptorSetLayout {
                &self.layout
            }

            fn new_cycle_elem(&self, device: &::sierra::Device) -> ::std::result::Result<#elem_ident, ::sierra::OutOfMemory> {
                ::std::result::Result::Ok(#elem_ident {
                    set: device.create_descriptor_set(::sierra::DescriptorSetInfo {
                        layout: self.layout.clone(),
                    })?,
                    #new_cycle_elem_descriptors
                    #new_cycle_elem_uniforms_buffer
                })
            }
        }

        impl ::sierra::DescriptorsInstance for #instance_ident {
            type Updated = #elem_ident;
            type Input = #ident;

            fn update<'a>(
                &'a mut self,
                input: &#ident,
                fence: usize,
                device: &::sierra::Device,
                writes: &mut impl ::std::iter::Extend<::sierra::WriteDescriptorSet<'a>>,
                encoder: &mut ::sierra::Encoder<'a>,
            ) -> ::std::result::Result<&'a #elem_ident, ::sierra::OutOfMemory> {
                self.update(input, fence, device, writes, encoder)
            }

            fn raw_layout(&self) -> &::sierra::DescriptorSetLayout {
                self.raw_layout()
            }
        }
    )
}
