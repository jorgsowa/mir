===description===
`$_FILES[x]['tmp_name'|'size'|'error']` are server-generated; `name`, `type` and `full_path` stay client-controlled.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function serverFields(): void {
    echo file_get_contents($_FILES['file']['tmp_name']);
    echo file_get_contents($_FILES['file']['tmp_name'][0]);
    $size = $_FILES['file']['size'];
    $error = $_FILES['file']['error'];
    echo file_get_contents($size . $error);
}

function clientFields(): void {
    echo file_get_contents($_FILES['file']['name']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    echo file_get_contents($_FILES['file']['full_path']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    echo file_get_contents($_FILES['file']['type']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}

// Only the field name directly under a file entry is server-generated.
function formFieldNamedLikeServerField(): void {
    echo file_get_contents($_FILES['tmp_name']['name']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}

function otherSuperglobals(): void {
    echo file_get_contents($_POST['tmp_name']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    echo file_get_contents($_POST['file']['tmp_name']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}
