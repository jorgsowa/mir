===description===
A native-only child return hint may narrow the parent's docblock refinement (`array` for `array{...}|null`).
===file===
<?php
/**
 * @template TValue
 * @template TView
 */
interface Transformer {
    /**
     * @param TValue|null $value
     * @return TView|null
     */
    public function transform(mixed $value): mixed;
}

/** @implements Transformer<string, array{year: int|null, week: int|null}> */
final class WeekTransformer implements Transformer {
    public function transform(mixed $value): array {
        return ['year' => null, 'week' => null];
    }
}
