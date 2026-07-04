===description===
`interface-string` and `interface-string<T>` parse from docblocks into
Atomic::TInterfaceString, matching class-string's parsing shape.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Shape {}

/** @param interface-string $a */
function bare(string $a): void {
    /** @mir-check $a is interface-string */
    $_ = $a;
}

/** @param interface-string<Shape> $b */
function bound(string $b): void {
    /** @mir-check $b is interface-string<Shape> */
    $_ = $b;
}

function var_annotation(string $c): void {
    /**
     * @var interface-string<Shape> $c
     * @mir-check $c is interface-string<Shape>
     */
    $_ = $c;
}
===expect===
