===description===
Opening a file or directory through SPL objects is a path sink.
`unserialize()` with `allowed_classes => false` can't create objects, so it
isn't an object injection sink. A `copy()` destination is a path too.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function open(): void {
    new SplFileObject($_GET['path']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    new DirectoryIterator($_GET['dir']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    new RecursiveDirectoryIterator($_GET['dir']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    copy('/tmp/a', $_GET['dest']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    new SplFileObject('/var/log/app.log');
}

function decode(): void {
    unserialize($_COOKIE['state'], ['allowed_classes' => false]);
    unserialize($_COOKIE['state']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
    unserialize($_COOKIE['state'], ['allowed_classes' => true]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
}
