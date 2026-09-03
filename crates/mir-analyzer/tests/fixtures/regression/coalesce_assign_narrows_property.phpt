===description===
FP: `$this->prop ??= $default` should leave a nullable property narrowed to
non-null afterwards, the same way `$var ??= $default` already narrows a
nullable local variable. Previously the coalesce-assign narrowing only
updated the flow-state type for simple variable targets, so property (and
static property) targets kept their stale nullable type and a subsequent
`return` was flagged as nullable even though it can never actually be null.
===file===
<?php
class Foo {
    public ?string $name = null;

    public function getName(): string {
        $this->name ??= 'default';
        return $this->name;
    }
}

class Bar {
    public static ?string $name = null;

    public static function getName(): string {
        self::$name ??= 'default';
        return self::$name;
    }
}
===expect===
