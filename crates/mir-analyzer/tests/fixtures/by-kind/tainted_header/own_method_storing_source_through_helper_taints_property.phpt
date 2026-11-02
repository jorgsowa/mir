===description===
A call to an own method that takes no tainted argument but stores source data in
`$this`'s properties, itself or through a helper it passes the data to, leaves those
properties tainted for the rest of the calling method and the methods it calls.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Request {
    private array $vars = [];
    private array $clean = [];

    private function parse(): void {
        $this->fill($_GET);
    }

    private function fill(array $source): void {
        foreach ($source as $key => $value) {
            $this->vars[$key] = $value;
        }
    }

    private function defaults(): void {
        $this->fill(['a' => 'fixed']);
    }

    private function parseTwice(): void {
        $this->parse();
    }

    private function send(): void {
        header('X-A: ' . $this->vars['a']);
    }

    public function main(): void {
        $this->parse();
        header('X-A: ' . $this->vars['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }

    public function viaSend(): void {
        $this->parse();
        $this->send();
//      ^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }

    public function twoHops(): void {
        $this->parseTwice();
        header('X-A: ' . $this->vars['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }

    public function clean(): void {
        $this->defaults();
        header('X-A: ' . $this->clean['a']);
        header('X-A: ' . $this->vars['a']);
    }
}
