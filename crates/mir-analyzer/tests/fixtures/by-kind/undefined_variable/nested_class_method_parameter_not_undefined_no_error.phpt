===description===
nested class method parameter not undefined no error
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outer(): void {
    class Inner {
        public function process(string $data): string {
            return $data;
        }
    }
}
