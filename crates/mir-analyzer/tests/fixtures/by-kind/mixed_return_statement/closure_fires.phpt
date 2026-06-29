===description===
A closure with a declared return type also fires MixedReturnStatement when returning a mixed value
===config===
suppress=UnusedVariable
===file===
<?php
$fn = function (): string {
    return json_decode('{}');
};
===expect===
MixedReturnStatement@3:4-3:29: Cannot return a mixed type from function with declared return type 'string'
