===description===
Calls that open the path they receive.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
chgrp($_GET['x'], 'g');
//<^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
finfo_file(finfo_open(), $_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
//         ^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $finfo of finfo_file() expects 'finfo', possibly different type 'finfo|false' provided
exif_read_data($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
exif_thumbnail($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
imagecreatefrompng($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
imagecreatefromjpeg($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
imagecreatefromstring($_GET['x']);
chgrp('/a', $_GET['x']);
