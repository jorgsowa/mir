===description===
The `unset` docblock pseudo-type is the null state: a method call on it is possibly null,
an `isset()` guard is not redundant, and `unset` is never looked up as a class.
===file===
<?php
/** @var \DateTime|unset $read */
echo $read->format('Y');
//   ^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method format() on possibly null value

/** @var \DateTime|unset $guarded */
if (isset($guarded)) {
    echo $guarded->format('Y');
}

function source(): mixed {
    return null;
}

/** @var \DateTime|unset $assigned */
$assigned = source();
echo $assigned->format('Y');
//   ^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method format() on possibly null value

/** @var DateTime|unset $checked */
$checked = source();
if (isset($checked)) {
    echo $checked->format('Y');
}
