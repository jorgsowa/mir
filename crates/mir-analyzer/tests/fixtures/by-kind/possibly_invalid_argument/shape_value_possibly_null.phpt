===description===
An array shape that fits the parameter once `null` is removed from its values is `PossiblyInvalidArgument`.
===file===
<?php
/** @param array{body: string, headers?: array<string, string>} $e */
function decode(array $e): void {
    echo $e['body'];
}
function f(?string $body): void {
    decode(['body' => $body]);
//         ^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $e of decode() expects 'array{'body': string, 'headers'?: array<string, string>}', possibly different type 'array{'body': string|null}' provided
}
