===description===
`unset` in a docblock union is read as `null`: the variable may hold nothing, so an unguarded read is possibly null and an `isset()` guard is not redundant.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @var \DateTime|unset $read */
echo $read->format('Y-m-d');
//   ^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method format() on possibly null value

/** @var \DateTime|unset $guarded */
if (isset($guarded)) {
    echo $guarded->format('Y-m-d');
    echo date_format($guarded, 'Y-m-d');
}

/** @param int|unset $value */
function takesOptional($value = null): void {
    /** @mir-check $value is int|null */
    echo 1;
}
