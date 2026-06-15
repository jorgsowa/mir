===description===
Prevent possibly undefined var in try
===config===
suppress=MissingThrowsDocblock,UnusedVariable
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
}
===expect===
PossiblyUndefinedVariable@22:9-22:11: Variable $a might not be defined
