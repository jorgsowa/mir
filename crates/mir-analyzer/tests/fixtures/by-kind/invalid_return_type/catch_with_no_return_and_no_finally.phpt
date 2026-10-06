===description===
Catch with no return and no finally
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo() : bool {
//               ^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
    try {
        if (rand(0, 1)) throw new Exception("bad");
        return true;
    } catch (Exception $e) {
        echo $e->getMessage();
        // do nothing here either
    }
}
