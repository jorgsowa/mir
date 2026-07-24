===description===
Negated form (`@psalm-assert-if-false !null $arr['key']`) targeting a
specific array key — proves the negation path (which reads the key's own
current value via `get_shape_path_type`, not the whole container) also
works for the array-key assertion target, not just the whole-parameter
case.
===config===
suppress=MissingReturnType,MixedArgument
===file===
<?php
/**
 * @param array{value?: ?string} $config
 * @psalm-assert-if-false !null $config['value']
 */
function lacksValue(array $config) {
    return !isset($config['value']);
}

/** @param array{value?: ?string} $c */
function test(array $c): void {
    if (!lacksValue($c)) {
        /** @mir-check $c is array{value: string} */
        $_ = 1;
    }
}
===expect===
