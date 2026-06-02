===description===
Catch with no return and no finally
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
InvalidReturnType@2:23-10:24: Return type 'void' is not compatible with declared 'bool'
