===description===
Catch with no return and no finally
===config===
suppress=MissingThrowsDocblock
===file===
<?php
function foo() : bool {
    try {
        if (rand(0, 1)) throw new Exception("bad");
        return true;
    } catch (Exception $e) {
        echo $e->getMessage();
        // do nothing here either
    }
}
===expect===
InvalidReturnType@2:22-10:1: Return type 'void' is not compatible with declared 'bool'
