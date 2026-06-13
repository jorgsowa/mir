===description===
Tainted llm prompt from concatenated input
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
class LlmAgent {
    /** @taint-sink llm_prompt $prompt */
    public function prompt(string $prompt): string {
        return "";
    }
}

$agent = new LlmAgent();
$agent->prompt("Tell me about " . (string) $_GET["topic"]);
===expect===
TaintedLlmPrompt@10:1-10:59: Tainted LLM prompt — possible prompt injection
