===description===
An inline @var naming the enclosing method's @template resolves inside a closure or arrow function; an undeclared name is still flagged
===file===
<?php
namespace App;

interface Named {}

class Registry {
    /**
     * @template T of Named
     * @param class-string<T> $name
     * @return T
     */
    public function get(string $name): Named {
        return new $name();
    }
}

class Builder {
    /** @template T of Named */
    public static function closure(Registry $r): callable {
        return static function (string $name) use ($r): ?Named {
            /** @var class-string<T> $name */
            return $r->get($name);
        };
    }

    /** @template T of Named */
    public static function arrow(Registry $r): callable {
        return fn(string $name): ?Named => $r->get($name);
    }

    /** @template T of Named */
    public static function nested(Registry $r): callable {
        return static function () use ($r): callable {
            return static function (string $name) use ($r): ?Named {
                /** @var class-string<T> $name */
                return $r->get($name);
            };
        };
    }

    public static function undeclared(Registry $r): callable {
        return static function (string $name) use ($r): ?Named {
            /** @var class-string<T> $name */
            return $r->get($name);
//                 ^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'App\T' does not satisfy bound 'App\Named'
        };
    }
}

/** @template T of Named */
function in_function(Registry $r): callable {
    return static function (string $name) use ($r): ?Named {
        /** @var class-string<T> $name */
        return $r->get($name);
    };
}
===expect===
