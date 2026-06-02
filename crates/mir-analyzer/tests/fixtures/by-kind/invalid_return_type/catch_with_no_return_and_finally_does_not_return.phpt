===description===
Catch with no return and finally does not return
===file===
<?php
function foo() : bool {
    try {
        if (rand(0, 1)) throw new Exception("bad");
        return true;
    } catch (Exception $e) {
        echo $e->getMessage();
        // do nothing here either
    } finally {

    }
}
===expect===
InvalidReturnType@2:23-12:24: Return type 'void' is not compatible with declared 'bool'
