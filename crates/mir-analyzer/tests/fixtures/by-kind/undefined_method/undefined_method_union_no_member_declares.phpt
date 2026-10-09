===description===
No union member declares the method: stays an Error on every atom.
===file===
<?php
class One {}
class Two {}
function test(One|Two $x): void {
    $x->missing();
//  ^^^^^^^^^^^^^ UndefinedMethod: Method One::missing() does not exist
}
