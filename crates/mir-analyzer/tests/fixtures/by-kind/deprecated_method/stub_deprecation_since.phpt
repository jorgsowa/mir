===description===
A built-in method is deprecated only from the PHP version its stub names; ArrayObject keeps the sort methods ArrayIterator loses in 8.6.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function sortAll(ArrayIterator $it, ArrayObject $ao, SplFileObject $file): void {
    $it->asort();
//  ^^^^^^^^^^^^ DeprecatedMethod: Method ArrayIterator::asort() is deprecated
    $ao->asort();
    $file->fgetcsv();
//  ^^^^^^^^^^^^^^^^ DeprecatedMethod: Method SplFileObject::fgetcsv() is deprecated
}
