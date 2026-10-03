===description===
catch type cross file missing
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Exceptions.php===
<?php
namespace App;
class RealException extends \Exception {}
===file:Handler.php===
<?php
use App\RealException;
use App\MissingException;
function handle(): void {
    try {
        throw new \Exception();
    } catch (RealException $e) {
    } catch (MissingException $e) {
//           ^^^^^^^^^^^^^^^^ UndefinedClass: Class App\MissingException does not exist
    }
}
===expect===
