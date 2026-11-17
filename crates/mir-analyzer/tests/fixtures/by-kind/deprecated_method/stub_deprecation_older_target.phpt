===description===
A built-in method deprecated in PHP 8.5 or 8.6 is reported for an 8.4 target too.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function track(SplObjectStorage $s, object $o, ArrayIterator $it): void {
    $s->attach($o);
//  ^^^^^^^^^^^^^^ DeprecatedMethod: Method SplObjectStorage::attach() is deprecated
    $it->asort();
//  ^^^^^^^^^^^^ DeprecatedMethod: Method ArrayIterator::asort() is deprecated
}
