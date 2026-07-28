===description===
`class-string<T>` returned from a templated static method and used as a
chained `::` receiver: T must substitute to the caller's bound class, same
as class_string_self_receiver_chained_static_call.phpt's `self` case — this
was already correct, kept here as a regression lock so it isn't broken while
fixing the sibling `self`/`static`/`parent` cases.
===file===
<?php

class Base {}
class Concrete extends Base {}

class Registry {
    /**
     * @template T of Base
     * @param class-string<T> $cls
     * @return class-string<T>
     */
    public static function register(string $cls): string {
        return $cls;
    }
}

Registry::register(Concrete::class)::doesNotExist();
===expect===
UndefinedMethod@17:0-17:51: Method Concrete::doesNotExist() does not exist
