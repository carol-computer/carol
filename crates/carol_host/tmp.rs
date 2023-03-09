#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::path::Path;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
mod host_bindings {
    use rand::RngCore;
    use wasmtime::component::bindgen;
    #[allow(clippy::all)]
    pub mod http {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        #[component(record)]
        pub struct Response {
            #[component(name = "status")]
            pub status: u16,
            #[component(name = "body")]
            pub body: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Response {
            #[inline]
            fn clone(&self) -> Response {
                Response {
                    status: ::core::clone::Clone::clone(&self.status),
                    body: ::core::clone::Clone::clone(&self.body),
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
                    &self.status,
                    memory,
                    <u16 as wasmtime::component::ComponentType>::ABI
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
        unsafe impl wasmtime::component::Lift for Response {
            #[inline]
            fn lift(
                store: &wasmtime::component::__internal::StoreOpaque,
                options: &wasmtime::component::__internal::Options,
                src: &Self::Lower,
            ) -> wasmtime::component::__internal::anyhow::Result<Self> {
                Ok(Self {
                    status: <u16 as wasmtime::component::Lift>::lift(
                        store,
                        options,
                        &src.status,
                    )?,
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
                    status: <u16 as wasmtime::component::Lift>::load(
                        memory,
                        &bytes[<u16 as wasmtime::component::ComponentType>::ABI
                            .next_field32_size(
                                &mut offset,
                            )..][..<u16 as wasmtime::component::ComponentType>::SIZE32],
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
            pub struct LowerResponse<T0: Copy, T1: Copy> {
                status: T0,
                body: T1,
                _align: [wasmtime::ValRaw; 0],
            }
            #[automatically_derived]
            impl<
                T0: ::core::clone::Clone + Copy,
                T1: ::core::clone::Clone + Copy,
            > ::core::clone::Clone for LowerResponse<T0, T1> {
                #[inline]
                fn clone(&self) -> LowerResponse<T0, T1> {
                    LowerResponse {
                        status: ::core::clone::Clone::clone(&self.status),
                        body: ::core::clone::Clone::clone(&self.body),
                        _align: ::core::clone::Clone::clone(&self._align),
                    }
                }
            }
            #[automatically_derived]
            impl<
                T0: ::core::marker::Copy + Copy,
                T1: ::core::marker::Copy + Copy,
            > ::core::marker::Copy for LowerResponse<T0, T1> {}
            unsafe impl wasmtime::component::ComponentType for Response {
                type Lower = LowerResponse<
                    <u16 as wasmtime::component::ComponentType>::Lower,
                    <Vec<u8> as wasmtime::component::ComponentType>::Lower,
                >;
                const ABI: wasmtime::component::__internal::CanonicalAbiInfo = wasmtime::component::__internal::CanonicalAbiInfo::record_static(
                    &[
                        <u16 as wasmtime::component::ComponentType>::ABI,
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
                                "status",
                                <u16 as wasmtime::component::ComponentType>::typecheck,
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
        impl core::fmt::Debug for Response {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Response")
                    .field("status", &self.status)
                    .field("body", &self.body)
                    .finish()
            }
        }
        const _: () = {
            if !(12 == <Response as wasmtime::component::ComponentType>::SIZE32) {
                ::core::panicking::panic(
                    "assertion failed: 12 == <Response as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(4 == <Response as wasmtime::component::ComponentType>::ALIGN32) {
                ::core::panicking::panic(
                    "assertion failed: 4 == <Response as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        pub trait Host: Sized {
            fn http_get(&mut self, url: String) -> wasmtime::Result<Response>;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            U: Host,
        {
            let mut inst = linker.instance("http")?;
            inst.func_wrap(
                "http-get",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (String,)| {
                    let host = get(caller.data_mut());
                    let r = host.http_get(arg0);
                    Ok((r?,))
                },
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod global {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub trait Host: Sized {
            fn bls_static_pubkey(&mut self) -> wasmtime::Result<Vec<u8>>;
            fn bls_static_sign(&mut self, message: Vec<u8>) -> wasmtime::Result<Vec<u8>>;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            U: Host,
        {
            let mut inst = linker.instance("global")?;
            inst.func_wrap(
                "bls-static-pubkey",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                    let host = get(caller.data_mut());
                    let r = host.bls_static_pubkey();
                    Ok((r?,))
                },
            )?;
            inst.func_wrap(
                "bls-static-sign",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (Vec<u8>,)| {
                    let host = get(caller.data_mut());
                    let r = host.bls_static_sign(arg0);
                    Ok((r?,))
                },
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod log {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub trait Host: Sized {
            fn info(&mut self, message: String) -> wasmtime::Result<()>;
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            U: Host,
        {
            let mut inst = linker.instance("log")?;
            inst.func_wrap(
                "info",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (String,)| {
                    let host = get(caller.data_mut());
                    let r = host.info(arg0);
                    r
                },
            )?;
            Ok(())
        }
    }
    #[allow(clippy::all)]
    pub mod contract {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub struct Contract {
            run: wasmtime::component::Func,
        }
        impl Contract {
            pub fn new(
                __exports: &mut wasmtime::component::ExportInstance<'_, '_>,
            ) -> wasmtime::Result<Contract> {
                let run = *__exports
                    .typed_func::<(&[u8], &[u8]), (Vec<u8>,)>("run")?
                    .func();
                Ok(Contract { run })
            }
            pub fn call_run<S: wasmtime::AsContextMut>(
                &self,
                mut store: S,
                arg0: &[u8],
                arg1: &[u8],
            ) -> wasmtime::Result<Vec<u8>> {
                let callee = unsafe {
                    wasmtime::component::TypedFunc::<
                        (&[u8], &[u8]),
                        (Vec<u8>,),
                    >::new_unchecked(self.run)
                };
                let (ret0,) = callee.call(store.as_context_mut(), (arg0, arg1))?;
                callee.post_return(store.as_context_mut())?;
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
                U: http::Host + global::Host + log::Host,
            {
                http::add_to_linker(linker, get)?;
                global::add_to_linker(linker, get)?;
                log::add_to_linker(linker, get)?;
                Ok(())
            }
            /// Instantiates the provided `module` using the specified
            /// parameters, wrapping up the result in a structure that
            /// translates between wasm and the host.
            pub fn instantiate<T>(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                component: &wasmtime::component::Component,
                linker: &wasmtime::component::Linker<T>,
            ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
                let instance = linker.instantiate(&mut store, component)?;
                Ok((Self::new(store, &instance)?, instance))
            }
            /// Instantiates a pre-instantiated module using the specified
            /// parameters, wrapping up the result in a structure that
            /// translates between wasm and the host.
            pub fn instantiate_pre<T>(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                instance_pre: &wasmtime::component::InstancePre<T>,
            ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
                let instance = instance_pre.instantiate(&mut store)?;
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
    const _: &str = "interface http {\n   record response {\n       status: u16,\n       body: list<u8>\n   }\n   http-get: func(url: string) -> response\n}\n\ninterface global {\n    bls-static-pubkey: func() -> list<u8>\n    bls-static-sign: func(message: list<u8>) -> list<u8>\n}\n\ninterface log {\n    info: func(message: string)\n}\n\ndefault world run-contract {\n  import http: self.http\n  import global: self.global\n  import log: self.log\n\n  export contract: interface {\n      run: func(contract-params: list<u8>, exec-input: list<u8>) -> list<u8>\n  }\n}\n";
    pub struct Host {
        pub bls_keypair: BlsKeyPair,
        pub contract_id: [u8; 32],
    }
    impl http::Http for Host {
        fn http_get(&mut self, url: String) -> anyhow::Result<http::Response> {
            let reqwest_response = reqwest::blocking::get(url)?;
            let response = http::Response {
                status: reqwest_response.status().as_u16(),
                body: reqwest_response.bytes()?.to_vec(),
            };
            Ok(response)
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
    impl global::Global for Host {
        fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
            Ok(self.bls_keypair.pk.to_uncompressed().to_vec())
        }
        fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
            use bls12_381::{
                hash_to_curve::{ExpandMsgXmd, HashToCurve},
                G2Projective, G2Affine,
            };
            let point = <G2Projective as HashToCurve<
                ExpandMsgXmd<sha2::Sha256>,
            >>::hash_to_curve(message, self.contract_id.as_ref());
            Ok(G2Affine::from(point * self.bls_keypair.sk).to_uncompressed().to_vec())
        }
    }
    impl log::Log for Host {
        fn info(&mut self, message: String) -> anyhow::Result<()> {
            {
                ::std::io::_print(format_args!("{0}\n", message));
            };
            Ok(())
        }
    }
}
use host_bindings::{RunContract, Host, BlsKeyPair};
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
            .run_contract()
            .call_run(&mut store, &contract_params, &exec_args)?;
        Ok(output)
    }
}
