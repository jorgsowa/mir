===description===
template narrowing with is_string, is_array, etc.
===file===
<?php
/**
 * @template TValue as string|array|int
 * @param TValue $value
 */
function handleMixed(string|array|int $value): void {
    if (is_string($value)) {
        echo strlen($value);
    } elseif (is_array($value)) {
        echo count($value);
    } else {
        echo $value * 2;
    }
}

/**
 * @template TValue as iterable
 * @param TValue $value
 */
function iterateValue(iterable $value): void {
    if (is_array($value)) {
        reset($value);
    } else {
        $value->current();
    }
}
===expect===
