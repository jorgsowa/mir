===description===
A built-in method deprecated in PHP 8.5 is not reported for an 8.4 target.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function track(SplObjectStorage $s, object $o, ArrayIterator $it): void {
    $s->attach($o);
    $it->asort();
}
