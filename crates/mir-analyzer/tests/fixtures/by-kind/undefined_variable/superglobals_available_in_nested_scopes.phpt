===description===
PHP superglobals ($_GET, $_POST, $_SERVER, ...) are implicitly available in
all scopes including nested closures and arrow functions — no UndefinedVariable.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outer(): void {
    isset($_GET['q']);

    $closure = function (): void {
        isset($_POST['name']);
    };

    $arrow = fn(): bool => isset($_SERVER['REQUEST_METHOD']);

    $closure();
    $arrow();
}
