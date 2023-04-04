#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::path::Path;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
mod host_bindings {
    use std::str::FromStr;
    use rand::RngCore;
    use wasmtime::component::bindgen;
    use async_trait::async_trait;
    #[allow(clippy::all)]
    pub mod http {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        #[component(record)]
        pub struct Response {
            #[component(name = "headers")]
            pub headers: Vec<(String, String)>,
            #[component(name = "body")]
            pub body: Vec<u8>,
            #[component(name = "status")]
            pub status: u16,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Response {
            #[inline]
            fn clone(&self) -> Response {
                Response {
                    headers: ::core::clone::Clone::clone(&self.headers),
                    body: ::core::clone::Clone::clone(&self.body),
                    status: ::core::clone::Clone::clone(&self.status),
                }
            }
        }
        unsafe impl wasmtime::component::Lower for Response {
            #[inline]
            fn lower<T>(
                &self,
                store: &mut wasmtime::StoreContextMut<T>,
                options: &wasmtime::component::__internal::Options,
                dst: &mut std::mem::MaybeUninit<Self::Lower>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                wasmtime::component::Lower::lower(
                    &self.headers,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).headers)
                            }
                        }
                    },
                )?;
                wasmtime::component::Lower::lower(
                    &self.body,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).body)
                            }
                        }
                    },
                )?;
                wasmtime::component::Lower::lower(
                    &self.status,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).status)
                            }
                        }
                    },
                )?;
                Ok(())
            }
            #[inline]
            fn store<T>(
                &self,
                memory: &mut wasmtime::component::__internal::MemoryMut<'_, T>,
                mut offset: usize,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                if true {
                    if !(offset
                        % (<Self as wasmtime::component::ComponentType>::ALIGN32
                            as usize) == 0)
                    {
                        ::core::panicking::panic(
                            "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                        )
                    }
                }
                wasmtime::component::Lower::store(
                    &self.headers,
                    memory,
                    <Vec<(String, String)> as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                wasmtime::component::Lower::store(
                    &self.body,
                    memory,
                    <Vec<u8> as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                wasmtime::component::Lower::store(
                    &self.status,
                    memory,
                    <u16 as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                Ok(())
            }
        }
        unsafe impl wasmtime::component::Lift for Response {
            #[inline]
            fn lift(
                store: &wasmtime::component::__internal::StoreOpaque,
                options: &wasmtime::component::__internal::Options,
                src: &Self::Lower,
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                Ok(Self {
                    headers: <Vec<
                        (String, String),
                    > as wasmtime::component::Lift>::lift(store, options, &src.headers)?,
                    body: <Vec<
                        u8,
                    > as wasmtime::component::Lift>::lift(store, options, &src.body)?,
                    status: <u16 as wasmtime::component::Lift>::lift(
                        store,
                        options,
                        &src.status,
                    )?,
                })
            }
            #[inline]
            fn load(
                memory: &wasmtime::component::__internal::Memory,
                bytes: &[u8],
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                if true {
                    if !((bytes.as_ptr() as usize)
                        % (<Self as wasmtime::component::ComponentType>::ALIGN32
                            as usize) == 0)
                    {
                        ::core::panicking::panic(
                            "assertion failed: (bytes.as_ptr() as usize) %\\n        (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                        )
                    }
                }
                let mut offset = 0;
                Ok(Self {
                    headers: <Vec<
                        (String, String),
                    > as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                    body: <Vec<
                        u8,
                    > as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<Vec<u8> as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<Vec<
                            u8,
                        > as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                    status: <u16 as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<u16 as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<u16 as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                })
            }
        }
        const _: () = {
            #[doc(hidden)]
            #[repr(C)]
            pub struct LowerResponse<T0: Copy, T1: Copy, T2: Copy> {
                headers: T0,
                body: T1,
                status: T2,
                _align: [wasmtime::ValRaw; 0],
            }
            #[automatically_derived]
            impl<
                T0: ::core::clone::Clone + Copy,
                T1: ::core::clone::Clone + Copy,
                T2: ::core::clone::Clone + Copy,
            > ::core::clone::Clone for LowerResponse<T0, T1, T2> {
                #[inline]
                fn clone(&self) -> LowerResponse<T0, T1, T2> {
                    LowerResponse {
                        headers: ::core::clone::Clone::clone(&self.headers),
                        body: ::core::clone::Clone::clone(&self.body),
                        status: ::core::clone::Clone::clone(&self.status),
                        _align: ::core::clone::Clone::clone(&self._align),
                    }
                }
            }
            #[automatically_derived]
            impl<
                T0: ::core::marker::Copy + Copy,
                T1: ::core::marker::Copy + Copy,
                T2: ::core::marker::Copy + Copy,
            > ::core::marker::Copy for LowerResponse<T0, T1, T2> {}
            unsafe impl wasmtime::component::ComponentType for Response {
                type Lower = LowerResponse<
                    <Vec<(String, String)> as wasmtime::component::ComponentType>::Lower,
                    <Vec<u8> as wasmtime::component::ComponentType>::Lower,
                    <u16 as wasmtime::component::ComponentType>::Lower,
                >;
                const ABI: wasmtime::component::__internal::CanonicalAbiInfo = wasmtime::component::__internal::CanonicalAbiInfo::record_static(
                    &[
                        <Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::ABI,
                        <Vec<u8> as wasmtime::component::ComponentType>::ABI,
                        <u16 as wasmtime::component::ComponentType>::ABI,
                    ],
                );
                #[inline]
                fn typecheck(
                    ty: &wasmtime::component::__internal::InterfaceType,
                    types: &wasmtime::component::__internal::ComponentTypes,
                ) -> wasmtime::component::__internal::anyhow::Result<()> {
                    wasmtime::component::__internal::typecheck_record(
                        ty,
                        types,
                        &[
                            (
                                "headers",
                                <Vec<
                                    (String, String),
                                > as wasmtime::component::ComponentType>::typecheck,
                            ),
                            (
                                "body",
                                <Vec<u8> as wasmtime::component::ComponentType>::typecheck,
                            ),
                            (
                                "status",
                                <u16 as wasmtime::component::ComponentType>::typecheck,
                            ),
                        ],
                    )
                }
            }
        };
        impl core::fmt::Debug for Response {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Response")
                    .field("headers", &self.headers)
                    .field("body", &self.body)
                    .field("status", &self.status)
                    .finish()
            }
        }
        const _: () = {
            if !(20 == <Response as wasmtime::component::ComponentType>::SIZE32) {
                ::core::panicking::panic(
                    "assertion failed: 20 == <Response as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(4 == <Response as wasmtime::component::ComponentType>::ALIGN32) {
                ::core::panicking::panic(
                    "assertion failed: 4 == <Response as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        #[component(enum)]
        pub enum Method {
            #[component(name = "get")]
            Get,
            #[component(name = "post")]
            Post,
            #[component(name = "put")]
            Put,
            #[component(name = "delete")]
            Delete,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Method {
            #[inline]
            fn clone(&self) -> Method {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Method {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Method {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Method {
            #[inline]
            fn eq(&self, other: &Method) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Method {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Method {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        unsafe impl wasmtime::component::Lower for Method {
            #[inline]
            fn lower<T>(
                &self,
                store: &mut wasmtime::StoreContextMut<T>,
                options: &wasmtime::component::__internal::Options,
                dst: &mut std::mem::MaybeUninit<Self::Lower>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                match self {
                    Self::Get => {
                        {
                            #[allow(unused_unsafe)]
                            {
                                unsafe {
                                    use ::wasmtime::component::__internal::MaybeUninitExt;
                                    let m: &mut std::mem::MaybeUninit<_> = dst;
                                    m.map(|p| &raw mut (*p).tag)
                                }
                            }
                        }
                            .write(wasmtime::ValRaw::u32(0u32));
                        unsafe {
                            wasmtime::component::__internal::lower_payload(
                                {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = dst;
                                            m.map(|p| &raw mut (*p).payload)
                                        }
                                    }
                                },
                                |payload| {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = payload;
                                            m.map(|p| &raw mut (*p).Get)
                                        }
                                    }
                                },
                                |dst| Ok(()),
                            )
                        }
                    }
                    Self::Post => {
                        {
                            #[allow(unused_unsafe)]
                            {
                                unsafe {
                                    use ::wasmtime::component::__internal::MaybeUninitExt;
                                    let m: &mut std::mem::MaybeUninit<_> = dst;
                                    m.map(|p| &raw mut (*p).tag)
                                }
                            }
                        }
                            .write(wasmtime::ValRaw::u32(1u32));
                        unsafe {
                            wasmtime::component::__internal::lower_payload(
                                {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = dst;
                                            m.map(|p| &raw mut (*p).payload)
                                        }
                                    }
                                },
                                |payload| {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = payload;
                                            m.map(|p| &raw mut (*p).Post)
                                        }
                                    }
                                },
                                |dst| Ok(()),
                            )
                        }
                    }
                    Self::Put => {
                        {
                            #[allow(unused_unsafe)]
                            {
                                unsafe {
                                    use ::wasmtime::component::__internal::MaybeUninitExt;
                                    let m: &mut std::mem::MaybeUninit<_> = dst;
                                    m.map(|p| &raw mut (*p).tag)
                                }
                            }
                        }
                            .write(wasmtime::ValRaw::u32(2u32));
                        unsafe {
                            wasmtime::component::__internal::lower_payload(
                                {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = dst;
                                            m.map(|p| &raw mut (*p).payload)
                                        }
                                    }
                                },
                                |payload| {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = payload;
                                            m.map(|p| &raw mut (*p).Put)
                                        }
                                    }
                                },
                                |dst| Ok(()),
                            )
                        }
                    }
                    Self::Delete => {
                        {
                            #[allow(unused_unsafe)]
                            {
                                unsafe {
                                    use ::wasmtime::component::__internal::MaybeUninitExt;
                                    let m: &mut std::mem::MaybeUninit<_> = dst;
                                    m.map(|p| &raw mut (*p).tag)
                                }
                            }
                        }
                            .write(wasmtime::ValRaw::u32(3u32));
                        unsafe {
                            wasmtime::component::__internal::lower_payload(
                                {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = dst;
                                            m.map(|p| &raw mut (*p).payload)
                                        }
                                    }
                                },
                                |payload| {
                                    #[allow(unused_unsafe)]
                                    {
                                        unsafe {
                                            use ::wasmtime::component::__internal::MaybeUninitExt;
                                            let m: &mut std::mem::MaybeUninit<_> = payload;
                                            m.map(|p| &raw mut (*p).Delete)
                                        }
                                    }
                                },
                                |dst| Ok(()),
                            )
                        }
                    }
                }
            }
            #[inline]
            fn store<T>(
                &self,
                memory: &mut wasmtime::component::__internal::MemoryMut<'_, T>,
                mut offset: usize,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                if true {
                    if !(offset
                        % (<Self as wasmtime::component::ComponentType>::ALIGN32
                            as usize) == 0)
                    {
                        ::core::panicking::panic(
                            "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                        )
                    }
                }
                match self {
                    Self::Get => {
                        *memory.get::<1usize>(offset) = 0u8.to_le_bytes();
                        Ok(())
                    }
                    Self::Post => {
                        *memory.get::<1usize>(offset) = 1u8.to_le_bytes();
                        Ok(())
                    }
                    Self::Put => {
                        *memory.get::<1usize>(offset) = 2u8.to_le_bytes();
                        Ok(())
                    }
                    Self::Delete => {
                        *memory.get::<1usize>(offset) = 3u8.to_le_bytes();
                        Ok(())
                    }
                }
            }
        }
        unsafe impl wasmtime::component::Lift for Method {
            #[inline]
            fn lift(
                store: &wasmtime::component::__internal::StoreOpaque,
                options: &wasmtime::component::__internal::Options,
                src: &Self::Lower,
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                Ok(
                    match src.tag.get_u32() {
                        0u32 => Self::Get,
                        1u32 => Self::Post,
                        2u32 => Self::Put,
                        3u32 => Self::Delete,
                        discrim => {
                            return ::anyhow::__private::Err(
                                ::anyhow::Error::msg({
                                    let res = ::alloc::fmt::format(
                                        format_args!("unexpected discriminant: {0}", discrim),
                                    );
                                    res
                                }),
                            );
                        }
                    },
                )
            }
            #[inline]
            fn load(
                memory: &wasmtime::component::__internal::Memory,
                bytes: &[u8],
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                let align = <Self as wasmtime::component::ComponentType>::ALIGN32;
                if true {
                    if !((bytes.as_ptr() as usize) % (align as usize) == 0) {
                        ::core::panicking::panic(
                            "assertion failed: (bytes.as_ptr() as usize) % (align as usize) == 0",
                        )
                    }
                }
                let discrim = bytes[0];
                let payload_offset = <Self as wasmtime::component::__internal::ComponentVariant>::PAYLOAD_OFFSET32;
                let payload = &bytes[payload_offset..];
                Ok(
                    match discrim {
                        0u8 => Self::Get,
                        1u8 => Self::Post,
                        2u8 => Self::Put,
                        3u8 => Self::Delete,
                        discrim => {
                            return ::anyhow::__private::Err(
                                ::anyhow::Error::msg({
                                    let res = ::alloc::fmt::format(
                                        format_args!("unexpected discriminant: {0}", discrim),
                                    );
                                    res
                                }),
                            );
                        }
                    },
                )
            }
        }
        const _: () = {
            #[doc(hidden)]
            #[repr(C)]
            pub struct LowerMethod {
                tag: wasmtime::ValRaw,
                payload: LowerPayloadMethod,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for LowerMethod {
                #[inline]
                fn clone(&self) -> LowerMethod {
                    let _: ::core::clone::AssertParamIsClone<wasmtime::ValRaw>;
                    let _: ::core::clone::AssertParamIsClone<LowerPayloadMethod>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for LowerMethod {}
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[repr(C)]
            union LowerPayloadMethod {
                Get: [wasmtime::ValRaw; 0],
                Post: [wasmtime::ValRaw; 0],
                Put: [wasmtime::ValRaw; 0],
                Delete: [wasmtime::ValRaw; 0],
            }
            #[automatically_derived]
            #[allow(non_snake_case)]
            impl ::core::clone::Clone for LowerPayloadMethod {
                #[inline]
                fn clone(&self) -> LowerPayloadMethod {
                    let _: ::core::clone::AssertParamIsCopy<Self>;
                    *self
                }
            }
            #[automatically_derived]
            #[allow(non_snake_case)]
            impl ::core::marker::Copy for LowerPayloadMethod {}
            unsafe impl wasmtime::component::ComponentType for Method {
                type Lower = LowerMethod;
                #[inline]
                fn typecheck(
                    ty: &wasmtime::component::__internal::InterfaceType,
                    types: &wasmtime::component::__internal::ComponentTypes,
                ) -> wasmtime::component::__internal::anyhow::Result<()> {
                    wasmtime::component::__internal::typecheck_enum(
                        ty,
                        types,
                        &["get", "post", "put", "delete"],
                    )
                }
                const ABI: wasmtime::component::__internal::CanonicalAbiInfo = wasmtime::component::__internal::CanonicalAbiInfo::variant_static(
                    &[None, None, None, None],
                );
            }
            unsafe impl wasmtime::component::__internal::ComponentVariant for Method {
                const CASES: &'static [Option<
                    wasmtime::component::__internal::CanonicalAbiInfo,
                >] = &[None, None, None, None];
            }
        };
        impl core::fmt::Debug for Method {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Method::Get => f.debug_tuple("Method::Get").finish(),
                    Method::Post => f.debug_tuple("Method::Post").finish(),
                    Method::Put => f.debug_tuple("Method::Put").finish(),
                    Method::Delete => f.debug_tuple("Method::Delete").finish(),
                }
            }
        }
        const _: () = {
            if !(1 == <Method as wasmtime::component::ComponentType>::SIZE32) {
                ::core::panicking::panic(
                    "assertion failed: 1 == <Method as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(1 == <Method as wasmtime::component::ComponentType>::ALIGN32) {
                ::core::panicking::panic(
                    "assertion failed: 1 == <Method as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        #[component(record)]
        pub struct Request {
            #[component(name = "method")]
            pub method: Method,
            #[component(name = "uri")]
            pub uri: String,
            #[component(name = "headers")]
            pub headers: Vec<(String, String)>,
            #[component(name = "body")]
            pub body: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Request {
            #[inline]
            fn clone(&self) -> Request {
                Request {
                    method: ::core::clone::Clone::clone(&self.method),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    headers: ::core::clone::Clone::clone(&self.headers),
                    body: ::core::clone::Clone::clone(&self.body),
                }
            }
        }
        unsafe impl wasmtime::component::Lower for Request {
            #[inline]
            fn lower<T>(
                &self,
                store: &mut wasmtime::StoreContextMut<T>,
                options: &wasmtime::component::__internal::Options,
                dst: &mut std::mem::MaybeUninit<Self::Lower>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                wasmtime::component::Lower::lower(
                    &self.method,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).method)
                            }
                        }
                    },
                )?;
                wasmtime::component::Lower::lower(
                    &self.uri,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).uri)
                            }
                        }
                    },
                )?;
                wasmtime::component::Lower::lower(
                    &self.headers,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).headers)
                            }
                        }
                    },
                )?;
                wasmtime::component::Lower::lower(
                    &self.body,
                    store,
                    options,
                    {
                        #[allow(unused_unsafe)]
                        {
                            unsafe {
                                use ::wasmtime::component::__internal::MaybeUninitExt;
                                let m: &mut std::mem::MaybeUninit<_> = dst;
                                m.map(|p| &raw mut (*p).body)
                            }
                        }
                    },
                )?;
                Ok(())
            }
            #[inline]
            fn store<T>(
                &self,
                memory: &mut wasmtime::component::__internal::MemoryMut<'_, T>,
                mut offset: usize,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
                if true {
                    if !(offset
                        % (<Self as wasmtime::component::ComponentType>::ALIGN32
                            as usize) == 0)
                    {
                        ::core::panicking::panic(
                            "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                        )
                    }
                }
                wasmtime::component::Lower::store(
                    &self.method,
                    memory,
                    <Method as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                wasmtime::component::Lower::store(
                    &self.uri,
                    memory,
                    <String as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                wasmtime::component::Lower::store(
                    &self.headers,
                    memory,
                    <Vec<(String, String)> as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                wasmtime::component::Lower::store(
                    &self.body,
                    memory,
                    <Vec<u8> as wasmtime::component::ComponentType>::ABI
                        .next_field32_size(&mut offset),
                )?;
                Ok(())
            }
        }
        unsafe impl wasmtime::component::Lift for Request {
            #[inline]
            fn lift(
                store: &wasmtime::component::__internal::StoreOpaque,
                options: &wasmtime::component::__internal::Options,
                src: &Self::Lower,
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                Ok(Self {
                    method: <Method as wasmtime::component::Lift>::lift(
                        store,
                        options,
                        &src.method,
                    )?,
                    uri: <String as wasmtime::component::Lift>::lift(
                        store,
                        options,
                        &src.uri,
                    )?,
                    headers: <Vec<
                        (String, String),
                    > as wasmtime::component::Lift>::lift(store, options, &src.headers)?,
                    body: <Vec<
                        u8,
                    > as wasmtime::component::Lift>::lift(store, options, &src.body)?,
                })
            }
            #[inline]
            fn load(
                memory: &wasmtime::component::__internal::Memory,
                bytes: &[u8],
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                if true {
                    if !((bytes.as_ptr() as usize)
                        % (<Self as wasmtime::component::ComponentType>::ALIGN32
                            as usize) == 0)
                    {
                        ::core::panicking::panic(
                            "assertion failed: (bytes.as_ptr() as usize) %\\n        (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                        )
                    }
                }
                let mut offset = 0;
                Ok(Self {
                    method: <Method as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<Method as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<Method as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                    uri: <String as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<String as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<String as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                    headers: <Vec<
                        (String, String),
                    > as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                    body: <Vec<
                        u8,
                    > as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<Vec<u8> as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<Vec<
                            u8,
                        > as wasmtime::component::ComponentType>::SIZE32],
                    )?,
                })
            }
        }
        const _: () = {
            #[doc(hidden)]
            #[repr(C)]
            pub struct LowerRequest<T0: Copy, T1: Copy, T2: Copy, T3: Copy> {
                method: T0,
                uri: T1,
                headers: T2,
                body: T3,
                _align: [wasmtime::ValRaw; 0],
            }
            #[automatically_derived]
            impl<
                T0: ::core::clone::Clone + Copy,
                T1: ::core::clone::Clone + Copy,
                T2: ::core::clone::Clone + Copy,
                T3: ::core::clone::Clone + Copy,
            > ::core::clone::Clone for LowerRequest<T0, T1, T2, T3> {
                #[inline]
                fn clone(&self) -> LowerRequest<T0, T1, T2, T3> {
                    LowerRequest {
                        method: ::core::clone::Clone::clone(&self.method),
                        uri: ::core::clone::Clone::clone(&self.uri),
                        headers: ::core::clone::Clone::clone(&self.headers),
                        body: ::core::clone::Clone::clone(&self.body),
                        _align: ::core::clone::Clone::clone(&self._align),
                    }
                }
            }
            #[automatically_derived]
            impl<
                T0: ::core::marker::Copy + Copy,
                T1: ::core::marker::Copy + Copy,
                T2: ::core::marker::Copy + Copy,
                T3: ::core::marker::Copy + Copy,
            > ::core::marker::Copy for LowerRequest<T0, T1, T2, T3> {}
            unsafe impl wasmtime::component::ComponentType for Request {
                type Lower = LowerRequest<
                    <Method as wasmtime::component::ComponentType>::Lower,
                    <String as wasmtime::component::ComponentType>::Lower,
                    <Vec<(String, String)> as wasmtime::component::ComponentType>::Lower,
                    <Vec<u8> as wasmtime::component::ComponentType>::Lower,
                >;
                const ABI: wasmtime::component::__internal::CanonicalAbiInfo = wasmtime::component::__internal::CanonicalAbiInfo::record_static(
                    &[
                        <Method as wasmtime::component::ComponentType>::ABI,
                        <String as wasmtime::component::ComponentType>::ABI,
                        <Vec<
                            (String, String),
                        > as wasmtime::component::ComponentType>::ABI,
                        <Vec<u8> as wasmtime::component::ComponentType>::ABI,
                    ],
                );
                #[inline]
                fn typecheck(
                    ty: &wasmtime::component::__internal::InterfaceType,
                    types: &wasmtime::component::__internal::ComponentTypes,
                ) -> wasmtime::component::__internal::anyhow::Result<()> {
                    wasmtime::component::__internal::typecheck_record(
                        ty,
                        types,
                        &[
                            (
                                "method",
                                <Method as wasmtime::component::ComponentType>::typecheck,
                            ),
                            (
                                "uri",
                                <String as wasmtime::component::ComponentType>::typecheck,
                            ),
                            (
                                "headers",
                                <Vec<
                                    (String, String),
                                > as wasmtime::component::ComponentType>::typecheck,
                            ),
                            (
                                "body",
                                <Vec<u8> as wasmtime::component::ComponentType>::typecheck,
                            ),
                        ],
                    )
                }
            }
        };
        impl core::fmt::Debug for Request {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Request")
                    .field("method", &self.method)
                    .field("uri", &self.uri)
                    .field("headers", &self.headers)
                    .field("body", &self.body)
                    .finish()
            }
        }
        const _: () = {
            if !(28 == <Request as wasmtime::component::ComponentType>::SIZE32) {
                ::core::panicking::panic(
                    "assertion failed: 28 == <Request as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(4 == <Request as wasmtime::component::ComponentType>::ALIGN32) {
                ::core::panicking::panic(
                    "assertion failed: 4 == <Request as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        pub trait Host: Sized {
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn execute<'life0, 'async_trait>(
                &'life0 mut self,
                request: Request,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = wasmtime::Result<Response>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            T: Send,
            U: Host + Send,
        {
            let mut inst = linker.instance("http")?;
            inst.func_wrap_async(
                "execute",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (Request,)| Box::new(async move {
                    let host = get(caller.data_mut());
                    let r = host.execute(arg0).await;
                    Ok((r?,))
                }),
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod global {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub trait Host: Sized {
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn bls_static_pubkey<'life0, 'async_trait>(
                &'life0 mut self,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = wasmtime::Result<Vec<u8>>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn bls_static_sign<'life0, 'async_trait>(
                &'life0 mut self,
                message: Vec<u8>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = wasmtime::Result<Vec<u8>>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            T: Send,
            U: Host + Send,
        {
            let mut inst = linker.instance("global")?;
            inst.func_wrap_async(
                "bls-static-pubkey",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| Box::new(async move {
                    let host = get(caller.data_mut());
                    let r = host.bls_static_pubkey().await;
                    Ok((r?,))
                }),
            )?;
            inst.func_wrap_async(
                "bls-static-sign",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (Vec<u8>,)| Box::new(async move {
                    let host = get(caller.data_mut());
                    let r = host.bls_static_sign(arg0).await;
                    Ok((r?,))
                }),
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod log {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub trait Host: Sized {
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn info<'life0, 'async_trait>(
                &'life0 mut self,
                message: String,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = wasmtime::Result<()>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            T: Send,
            U: Host + Send,
        {
            let mut inst = linker.instance("log")?;
            inst.func_wrap_async(
                "info",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (String,)| Box::new(async move {
                    let host = get(caller.data_mut());
                    let r = host.info(arg0).await;
                    r
                }),
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod contract {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub struct Contract {
            activate: wasmtime::component::Func,
        }
        impl Contract {
            pub fn new(
                __exports: &mut wasmtime::component::ExportInstance<'_, '_>,
            ) -> wasmtime::Result<Contract> {
                let activate = *__exports
                    .typed_func::<(&[u8], &[u8]), (Vec<u8>,)>("activate")?
                    .func();
                Ok(Contract { activate })
            }
            pub async fn call_activate<S: wasmtime::AsContextMut>(
                &self,
                mut store: S,
                arg0: &[u8],
                arg1: &[u8],
            ) -> wasmtime::Result<Vec<u8>>
            where
                <S as wasmtime::AsContext>::Data: Send,
            {
                let callee = unsafe {
                    wasmtime::component::TypedFunc::<
                        (&[u8], &[u8]),
                        (Vec<u8>,),
                    >::new_unchecked(self.activate)
                };
                let (ret0,) = callee
                    .call_async(store.as_context_mut(), (arg0, arg1))
                    .await?;
                callee.post_return_async(store.as_context_mut()).await?;
                Ok(ret0)
            }
        }
    }
    pub struct RunContract {
        contract: contract::Contract,
    }
    const _: () = {
        use wasmtime::component::__internal::anyhow;
        impl RunContract {
            pub fn add_to_linker<T, U>(
                linker: &mut wasmtime::component::Linker<T>,
                get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
            ) -> wasmtime::Result<()>
            where
                U: http::Host + global::Host + log::Host + Send,
                T: Send,
            {
                http::add_to_linker(linker, get)?;
                global::add_to_linker(linker, get)?;
                log::add_to_linker(linker, get)?;
                Ok(())
            }
            /// Instantiates the provided `module` using the specified
            /// parameters, wrapping up the result in a structure that
            /// translates between wasm and the host.
            pub async fn instantiate_async<T: Send>(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                component: &wasmtime::component::Component,
                linker: &wasmtime::component::Linker<T>,
            ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
                let instance = linker.instantiate_async(&mut store, component).await?;
                Ok((Self::new(store, &instance)?, instance))
            }
            /// Instantiates a pre-instantiated module using the specified
            /// parameters, wrapping up the result in a structure that
            /// translates between wasm and the host.
            pub async fn instantiate_pre<T: Send>(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                instance_pre: &wasmtime::component::InstancePre<T>,
            ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
                let instance = instance_pre.instantiate_async(&mut store).await?;
                Ok((Self::new(store, &instance)?, instance))
            }
            /// Low-level creation wrapper for wrapping up the exports
            /// of the `instance` provided in this structure of wasm
            /// exports.
            ///
            /// This function will extract exports from the `instance`
            /// defined within `store` and wrap them all up in the
            /// returned structure which can be used to interact with
            /// the wasm module.
            pub fn new(
                mut store: impl wasmtime::AsContextMut,
                instance: &wasmtime::component::Instance,
            ) -> wasmtime::Result<Self> {
                let mut store = store.as_context_mut();
                let mut exports = instance.exports(&mut store);
                let mut __exports = exports.root();
                let contract = contract::Contract::new(
                    &mut __exports
                        .instance("contract")
                        .ok_or_else(|| ::anyhow::__private::must_use({
                            let error = ::anyhow::__private::format_err(
                                format_args!("exported instance `contract` not present"),
                            );
                            error
                        }))?,
                )?;
                Ok(RunContract { contract })
            }
            pub fn contract(&self) -> &contract::Contract {
                &self.contract
            }
        }
    };
    const _: &str = "interface http {\n    enum method {\n       get,\n       post,\n       put,\n       delete\n    }\n    record request {\n         method: method,\n         uri: string,\n         headers: list<tuple<string,string>>,\n         body: list<u8>,\n    }\n\n    record response {\n        headers: list<tuple<string,string>>,\n        body: list<u8>,\n        status: u16\n    }\n    execute: func(request: request) -> response\n}\n\ninterface global {\n    bls-static-pubkey: func() -> list<u8>\n    bls-static-sign: func(message: list<u8>) -> list<u8>\n}\n\ninterface log {\n    info: func(message: string)\n}\n\ndefault world run-contract {\n    import http: self.http\n    import global: self.global\n    import log: self.log\n\n    export contract: interface {\n        // use self.http.{request as http-request,response as http-response}\n        activate: func(contract-params: list<u8>, input: list<u8>) -> list<u8>\n    }\n}\n";
    pub struct Host {
        pub bls_keypair: BlsKeyPair,
        pub contract_id: [u8; 32],
        pub http_client: reqwest::blocking::Client,
    }
    impl TryFrom<http::Request> for reqwest::Request {
        type Error = anyhow::Error;
        fn try_from(guest_request: http::Request) -> Result<Self, Self::Error> {
            let uri = reqwest::Url::parse(&guest_request.uri)?;
            let req = reqwest::Request::new(guest_request.method.into(), uri);
            let headers = req.headers_mut();
            for (key, value) in guest_request.headers {
                let header_name = reqwest::header::HeaderName::from_str(&key)?;
                let header_value = reqwest::header::HeaderValue::from_str(&value)?;
                headers.append(header_name, header_value);
            }
            *req.body_mut() = Some(reqwest::Body::from(guest_request.body));
            Ok(req)
        }
    }
    impl From<http::Method> for reqwest::Method {
        fn from(value: http::Method) -> Self {
            use http::Method::*;
            match value {
                Get => reqwest::Method::GET,
                Post => reqwest::Method::POST,
                Put => reqwest::Method::PUT,
                Delete => reqwest::Method::DELETE,
            }
        }
    }
    impl http::Host for Host {
        #[allow(
            clippy::async_yields_async,
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn execute<'life0, 'async_trait>(
            &'life0 mut self,
            request: http::Request,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = anyhow::Result<http::Response>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret)
                    = ::core::option::Option::None::<anyhow::Result<http::Response>> {
                    return __ret;
                }
                let mut __self = self;
                let request = request;
                let __ret: anyhow::Result<http::Response> = {
                    let request = request.try_into()?;
                    let res = __self.http_client.execute(request)?;
                    let response = http::Response {
                        status: res.status().as_u16(),
                        body: res.bytes()?.to_vec(),
                        headers: res
                            .headers()
                            .into_iter()
                            .map(|(key, value)| (key.to_string(), value))
                            .collect(),
                    };
                    Ok(response)
                };
                #[allow(unreachable_code)] __ret
            })
        }
    }
    pub struct BlsKeyPair {
        pk: bls12_381::G1Affine,
        sk: bls12_381::Scalar,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for BlsKeyPair {
        #[inline]
        fn clone(&self) -> BlsKeyPair {
            BlsKeyPair {
                pk: ::core::clone::Clone::clone(&self.pk),
                sk: ::core::clone::Clone::clone(&self.sk),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BlsKeyPair {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "BlsKeyPair",
                "pk",
                &self.pk,
                "sk",
                &&self.sk,
            )
        }
    }
    impl BlsKeyPair {
        pub fn new(sk: bls12_381::Scalar) -> Self {
            let pk = bls12_381::G1Affine::generator() * &sk;
            Self { pk: pk.into(), sk }
        }
        pub fn random(rng: &mut impl RngCore) -> Self {
            let mut bytes = [0u8; 64];
            rng.fill_bytes(&mut bytes);
            let sk = bls12_381::Scalar::from_bytes_wide(&bytes);
            Self::new(sk)
        }
    }
    impl global::Host for Host {
        fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
            Ok(self.bls_keypair.pk.to_uncompressed().to_vec())
        }
        fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
            use bls12_381::{
                hash_to_curve::{ExpandMsgXmd, HashToCurve},
                G2Affine, G2Projective,
            };
            let point = <G2Projective as HashToCurve<
                ExpandMsgXmd<sha2::Sha256>,
            >>::hash_to_curve(message, self.contract_id.as_ref());
            Ok(G2Affine::from(point * self.bls_keypair.sk).to_uncompressed().to_vec())
        }
    }
    impl log::Host for Host {
        fn info(&mut self, message: String) -> anyhow::Result<()> {
            {
                ::std::io::_print(format_args!("{0}\n", message));
            };
            Ok(())
        }
    }
}
use host_bindings::{BlsKeyPair, Host, RunContract};
pub struct Executor {
    engine: Engine,
}
#[automatically_derived]
impl ::core::clone::Clone for Executor {
    #[inline]
    fn clone(&self) -> Executor {
        Executor {
            engine: ::core::clone::Clone::clone(&self.engine),
        }
    }
}
pub struct Contract {
    component: Component,
}
#[automatically_derived]
impl ::core::clone::Clone for Contract {
    #[inline]
    fn clone(&self) -> Contract {
        Contract {
            component: ::core::clone::Clone::clone(&self.component),
        }
    }
}
impl Executor {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("valid config");
        Self { engine }
    }
    pub fn load_contract_from_file(
        &self,
        file: impl AsRef<Path>,
    ) -> anyhow::Result<Contract> {
        Ok(Contract {
            component: Component::from_file(&self.engine, file)?,
        })
    }
    pub fn execute_contract(
        &self,
        contract: Contract,
        contract_params: Vec<u8>,
        exec_args: Vec<u8>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut linker = Linker::new(&self.engine);
        RunContract::add_to_linker(&mut linker, |state: &mut Host| state)?;
        let mut store = Store::new(
            &self.engine,
            Host {
                bls_keypair: BlsKeyPair::random(&mut rand::thread_rng()),
                contract_id: [0u8; 32],
            },
        );
        let (bindings, _) = RunContract::instantiate(
            &mut store,
            &contract.component,
            &linker,
        )?;
        let output = bindings
            .contract()
            .call_activate(&mut store, &contract_params, &exec_args)?;
        Ok(output)
    }
}
