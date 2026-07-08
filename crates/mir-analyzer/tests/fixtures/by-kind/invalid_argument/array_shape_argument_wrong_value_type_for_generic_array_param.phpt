===description===
A shape argument whose property values don't fit a generic array<K,V>
param's value type is flagged, per-property rather than as one merged
union
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param array<int,int> $x */
function needsIntArray(array $x): void {}
needsIntArray(['a' => 'b']);
===expect===
InvalidArgument@4:14-4:26: Argument $x of needsIntArray() expects 'array<int, int>', got 'array{'a': "b"}'
