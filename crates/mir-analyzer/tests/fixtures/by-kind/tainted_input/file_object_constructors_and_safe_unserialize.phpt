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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    new DirectoryIterator($_GET['dir']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    new RecursiveDirectoryIterator($_GET['dir']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    copy('/tmp/a', $_GET['dest']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    new SplFileObject('/var/log/app.log');
}

function decode(): void {
    unserialize($_COOKIE['state'], ['allowed_classes' => false]);
    unserialize($_COOKIE['state']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'unserialize'
    unserialize($_COOKIE['state'], ['allowed_classes' => true]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'unserialize'
}
