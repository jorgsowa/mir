===description===
`empty` (Psalm's falsy pseudo-type) is parsed as a real type instead of a
bogus named class; all falsy literals satisfy it as both a param and a
return type.
===config===
suppress=UnusedParam
===file===
<?php
/** @param empty $x */
function takeEmpty($x): void {}

takeEmpty(false);
takeEmpty(null);
takeEmpty(0);
takeEmpty(0.0);
takeEmpty("");
takeEmpty("0");
takeEmpty([]);

/** @return empty */
function returnsFalse() {
    return false;
}

/** @return empty */
function returnsEmptyArray() {
    return [];
}

===expect===
