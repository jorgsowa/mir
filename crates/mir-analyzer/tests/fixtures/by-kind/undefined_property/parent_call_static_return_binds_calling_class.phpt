===description===
`parent::m()` returning `static` keeps the caller's late static binding, so the result is the calling class, not the parent.
===file===
<?php
class Filter {
    public static function load(): static { return new static(); }
    public function copy(): static { return clone $this; }
    public function same(): self { return $this; }
}

final class LevelFilter extends Filter {
    public string $level = '';

    public static function load(): static {
        $filter = parent::load();
        $filter->level = 'error';
        return $filter;
    }

    public function copy(): static {
        $copy = parent::copy();
        echo $copy->level;
        return $copy;
    }

    public function sameLevel(): string {
        return parent::same()->level;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
//                             ^^^^^ UndefinedProperty: Property Filter::$level does not exist
    }
}
