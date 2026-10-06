===description===
Prevent possibly undefined var in try
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public static function possiblyThrows(): bool {
        $result = (bool)rand(0, 1);

        if (!$result) {
            throw new Exception("BOOM");
        }

        return true;
    }
}

try {
    $result = Foo::possiblyThrows();
    $a = "ACME";

    if ($result) {
        echo $a;
    }
} catch (Exception $e) {
    echo $a;
//       ^^ PossiblyUndefinedVariable: Variable $a might not be defined
}
