===description===
G1: property fetch on an unconstrained template param must not emit MixedPropertyFetch —
a bare T is an intentionally parameterised placeholder, not truly mixed.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template T
 * @param T $obj
 */
function process($obj): void {
    // T is a template param — must not fire MixedPropertyFetch
    $obj->name;
}
===expect===
