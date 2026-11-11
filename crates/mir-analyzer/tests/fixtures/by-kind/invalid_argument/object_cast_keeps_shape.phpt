===description===
`(object)` on an array shape is `object{...}&stdClass`: a mistyped property is rejected by an object-shape parameter. Its properties may be reassigned, so a later read is not typed by the cast, and an unlisted property is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param object{foo: int}&\stdClass $shape */
function takesWritable(object $shape): void {}

/** @param object{foo: int} $shape */
function takesShape(object $shape): void {}

function takesStdClass(stdClass $value): void {}

takesWritable((object) ['foo' => 1]);
takesShape((object) ['foo' => 1]);
takesStdClass((object) ['foo' => 1]);
takesWritable((object) ['foo' => 'wrong']);
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $shape of takesWritable() expects 'object{foo: int}&stdClass', got 'object{foo: "wrong"}&stdClass'
takesShape((object) ['foo' => 'wrong']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $shape of takesShape() expects 'object{foo: int}', got 'object{foo: "wrong"}&stdClass'

$point = (object) ['x' => 1, 'y' => 'top'];
$point->x = 'moved';
echo strlen($point->x);
echo $point->label ?? 'none';
$point->label = 'set';
