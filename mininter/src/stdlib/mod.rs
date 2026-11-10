mod array;
mod binary;
mod io;
mod math;

use crate::runtime::vm::Vm;

pub fn register_stdlib(vm: &mut Vm) {
    #[cfg(feature = "rand")]
    let rng = vm.rng();

    vm.register_native("char", array::char_fn::char_fn, 1);
    vm.register_native("code", array::code::code, 1);
    vm.register_native("len", array::len::len, 1);
    vm.register_native("slice", array::slice::slice, 3);
    vm.register_native("str", array::str::str, 1);

    vm.register_native("bitAnd", binary::bit_and::bit_and, 2);
    vm.register_native("bitOr", binary::bit_or::bit_or, 2);
    vm.register_native("bitXor", binary::bit_xor::bit_xor, 2);

    vm.register_native("err", io::err::err, 1);
    vm.register_native("input", io::input::input, 1);
    vm.register_native("print", io::print::print, 2);
    #[cfg(feature = "rand")]
    vm.register_native(
        "rnd",
        move |args| {
            let rng = rng.borrow_mut();
            io::rnd::rnd(args, rng)
        },
        1,
    );

    vm.register_native("abs", math::abs::abs, 1);
    vm.register_native("ceil", math::ceil::ceil, 1);
    vm.register_native("floor", math::floor::floor, 1);
    vm.register_native("pi", math::pi::pi, 0);
    vm.register_native("range", math::range_fn::range, 3);
    vm.register_native("round", math::round::round, 2);
    vm.register_native("sign", math::sign::sign, 1);
    vm.register_native("sqrt", math::sqrt::sqrt, 1);
    vm.register_native("acos", math::trig::acos::acos, 1);
    vm.register_native("asin", math::trig::asin::asin, 1);
    vm.register_native("atan", math::trig::atan::atan, 1);
    vm.register_native("cos", math::trig::cos::cos, 1);
    vm.register_native("log", math::trig::log::log, 1);
    vm.register_native("sin", math::trig::sin::sin, 1);
    vm.register_native("tan", math::trig::tan::tan, 1);
    vm.register_native("val", math::val::val, 1);
}
