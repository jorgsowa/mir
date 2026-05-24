===description===
catchWithNoReturnAndFinallyDoesNotReturn
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
InvalidReturnType
===ignore===
TODO
