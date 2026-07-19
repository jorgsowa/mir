===description===
`$var::method()` now binds the class-level `@template` from the receiver
variable's own concrete type args (e.g. `Box<int> $b`), matching what
`$b->method()` already did — a static method's `@return T`/`@param T` no
longer leaks the raw template atom or skips argument-type checking.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $seed */
    public static function make($seed): static { return new static(); }
    /** @return T */
    public static function peek() {
        return null;
    }
}

/** @param Box<int> $b */
function reads_bound_template(Box $b): void {
    $x = $b::peek();
    /** @mir-check $x is int */
    $_ = $x;
}

/** @param Box<int> $b */
function checks_bound_template(Box $b): void {
    $b::make('not an int');
}
===expect===
InvalidArgument@21:13-21:25: Argument $seed of make() expects 'int', got '"not an int"'
