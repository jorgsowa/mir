===description===
Constructor sinks apply to the stub constructor only. A subclass overriding
`__construct` (directly or through an intermediate class) takes its own
arguments, and `SplTempFileObject`'s first argument is a memory limit.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class BillingClient extends SoapClient {
    public function __construct(array $creds) {
        parent::__construct('https://fixed.example.com/x.wsdl', $creds);
    }
}

class CsvReader extends SplFileObject {
    public function __construct(string $delimiter) {
        parent::__construct('/var/data/fixed.csv');
        $this->setCsvControl($delimiter);
    }
}

class SubBilling extends BillingClient {}

class PlainSoap extends SoapClient {}
class DeepPlainSoap extends PlainSoap {}

class PlainFile extends SplFileObject {}

class OverridingDir extends DirectoryIterator {
    public function __construct(int $depth) {
        parent::__construct('/var/data/' . $depth);
    }
}

function overridden(): void {
    new BillingClient(['user' => $_POST['u']]);
    new SubBilling(['user' => $_POST['u']]);
    new CsvReader($_GET['d']);
    new OverridingDir($_GET['depth']);
    new SplTempFileObject($_GET['n']);
}

function inherited(): void {
    new PlainSoap($_GET['wsdl']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    new DeepPlainSoap($_GET['wsdl']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    new PlainFile($_GET['path']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    new FilesystemIterator($_GET['dir']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    new GlobIterator($_GET['pattern']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}
