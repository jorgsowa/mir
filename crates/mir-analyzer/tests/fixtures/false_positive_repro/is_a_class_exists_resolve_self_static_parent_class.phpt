===description===
`self::class` / `static::class` / `parent::class` must resolve as the
class-name argument to is_a()/is_subclass_of()/class_exists() — the
shared extract_class_fqcn_from_expr explicitly excluded self/static/parent
instead of resolving them via the caller's own known FQCNs, unlike its
sibling extract_class_name.
===config===
suppress=UnusedParam,MissingParamType
===file===
<?php
class Animal {}

class Dog extends Animal {
    public function checkSelf($x): void {
        if (is_a($x, self::class)) {
            /** @mir-check $x is Dog */
            $_ = 1;
        }
    }

    public function checkStatic($x): void {
        if (is_a($x, static::class)) {
            /** @mir-check $x is Dog */
            $_ = 1;
        }
    }
}

class Puppy extends Dog {
    public function checkParent($x): void {
        if (is_subclass_of($x, parent::class)) {
            /** @mir-check $x is Dog */
            $_ = 1;
        }
    }
}
===expect===
MixedArgument@6:17-6:19: Argument $object_or_class of is_a() is mixed
MixedArgument@13:17-13:19: Argument $object_or_class of is_a() is mixed
MixedArgument@22:27-22:29: Argument $object_or_class of is_subclass_of() is mixed
