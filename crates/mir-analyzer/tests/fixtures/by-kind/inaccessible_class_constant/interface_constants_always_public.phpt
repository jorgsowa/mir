===description===
InaccessibleClassConstant does NOT fire for interface constants, which are always public.
===config===
suppress=UnusedVariable,MixedAssignment
===file===
<?php
interface Limits {
    const MAX_ITEMS = 100;
    const MIN_ITEMS = 1;
}

$max = Limits::MAX_ITEMS;
$min = Limits::MIN_ITEMS;
===expect===
