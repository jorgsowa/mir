===description===
stub without @template annotations returns array<mixed, mixed>, not a typed list
===config===
stub_file=stubs/helpers.php
suppress=UnusedVariable,UnusedFunction
===file:stubs/helpers.php===
<?php
function array_key_list(array $array): array {}
===file:App.php===
<?php
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
    /** @mir-check $keys is list<string> */
    $_ = $keys;
}
===expect===
App.php: TypeCheckMismatch@5:5-5:16: Type of $keys is expected to be list<string>, got array<mixed, mixed>
