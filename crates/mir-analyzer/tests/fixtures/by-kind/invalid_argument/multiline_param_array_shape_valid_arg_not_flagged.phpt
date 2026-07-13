===description===
A valid array literal matching a multi-line @param array shape is not flagged.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param array{
 *     id: int,
 *     name: string,
 * } $data
 */
function f(array $data): void {}

f(["id" => 1, "name" => "Alice"]);
===expect===
