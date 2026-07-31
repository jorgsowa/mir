===description===
FP-K5: a class composing a trait that itself `use`s another trait declaring
a `final` method was flagged FinalMethodOverridden against its OWN flattened
copy — `own_traits` (used to recognize "this trait is already flattened
into own, not a real parent") only listed the DIRECTLY `use`d trait, so the
transitively-composed inner trait still looked like a real ancestor to
compare the final method against. Verified live: this loads with no fatal.
===file===
<?php
trait Inner {
    final public function f(): void {}
}
trait Outer {
    use Inner;
}
class C {
    use Outer;
}
===expect===
