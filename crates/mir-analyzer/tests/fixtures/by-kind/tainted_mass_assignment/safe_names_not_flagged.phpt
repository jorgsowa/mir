===description===
Names from the program, names checked against an allowlist, names confined by a
literal prefix, nested session keys, list spreads and spreads before PHP 8 are
not reported.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Settings {
    public string $theme = '';
    public string $field_a = '';
}

function createUser(string $name, string $email = ''): void {}

function allowlisted(Settings $s): void {
    foreach ($_POST as $key => $value) {
        if (in_array($key, ['theme', 'locale'], true)) {
            $s->$key = (string) $value;
        }
    }
}

function prefixed(Settings $s): void {
    $k = $_GET['k'];
    $s->{'field_' . $k} = 'x';
    $_SESSION['cart_' . $k] = 1;
    $_SESSION['cart'][$k] = 1;
}

function fromProgram(Settings $s, array $config): void {
    foreach ($config as $key => $value) {
        $s->$key = $value;
    }
    $_SESSION['user'] = $_GET['name'];
}

function positional(): void {
    createUser(...array_values($_POST));
    $args = [$_GET['name'], $_GET['email']];
    createUser(...$args);
}
