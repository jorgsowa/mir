===description===
FP: static factory method with method-level @template T should bind T to the argument type
(e.g. mixed from array access). Before the fix, T was left as the literal template name,
producing Option<T>|Option<never> instead of Option<mixed>|Option<never>, which
then failed the InvalidPropertyAssignment check against Option<string|null>.
===file===
<?php

/**
 * @template TValue
 */
class Option {
    /**
     * @template T
     * @param T $value
     * @return Option<T>
     */
    public static function some(mixed $value): self {
        unset($value);
        return new self();
    }

    /**
     * @return Option<never>
     */
    public static function none(): self {
        return new self();
    }
}

class Dto {
    /** @var Option<string|null> */
    private Option $label;

    /**
     * @param array<string, mixed> $args
     */
    public function __construct(array $args) {
        $this->label = array_key_exists('label', $args)
            ? Option::some($args['label'])
            : Option::none();
    }
}
===expect===
UnusedParam@12:33-12:45: Parameter $value is never used
