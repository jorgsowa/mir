===description===
PHP 8.6 deprecates ArrayIterator's sort methods and SplFileObject's CSV methods; ArrayObject keeps its sort methods.
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
