===description===
A shape argument whose property values DO fit a generic array<K,V>
param's value type is accepted — control case for the fix that stopped
treating every shape as unconditionally array-compatible
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param array<int,int> $x */
function needsIntArray(array $x): void {}
needsIntArray([1, 2, 3]);
===expect===
