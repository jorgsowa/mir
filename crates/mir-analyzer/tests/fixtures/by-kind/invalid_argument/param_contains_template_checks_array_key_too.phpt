===description===
`param_contains_template_or_unknown`'s array arm only inspected the VALUE
type for an unresolved template/unknown class, never the KEY — a
`@template TKey of array-key` used only in `array<TKey, V>`'s key position
left the whole array param treated as fully concrete, so an argument that
doesn't structurally look like an array at all got a false InvalidArgument
instead of being forgiven the same way a value-side template already was.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @template TKey of array-key
 * @param array<TKey, string> $map
 */
function needsMap($map): void {}

function test_key_only_template_is_forgiven(): void {
    needsMap(5);
}

/** @param array<int, string> $map */
function needsConcreteMap($map): void {}

function test_concrete_array_param_still_flagged(): void {
    needsConcreteMap(5);
}
===expect===
InvalidArgument@16:21-16:22: Argument $map of needsConcreteMap() expects 'array<int, string>', got '5'
