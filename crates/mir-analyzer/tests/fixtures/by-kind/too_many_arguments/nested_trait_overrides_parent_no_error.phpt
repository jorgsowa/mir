===description===
Sub-trait method (trait-within-a-trait) that overrides a parent class method
with additional optional params must not report TooManyArguments. This is
the exact shape of the Carbon bug: CarbonImmutable extends DateTimeImmutable
and uses Date, which in turn uses Timestamp; Timestamp::createFromTimestamp
adds an optional $timezone param that the DateTimeImmutable stub lacks.
===file===
<?php
class ParentClass {
    public string $label = '';

    public static function create(string $value): static {
        $i = new static();
        $i->label = $value;
        return $i;
    }
}

trait InnerTrait {
    public static function create(string $value, ?string $context = null): static {
        $i = new static();
        $i->label = $context !== null ? $value . ':' . $context : $value;
        return $i;
    }
}

trait OuterTrait {
    use InnerTrait;
}

class Child extends ParentClass {
    use OuterTrait;
}

// Two args valid per InnerTrait::create — must not raise TooManyArguments.
Child::create('foo', 'bar');
===expect===
