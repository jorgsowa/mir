===description===
taintedLlmPromptFromUserInput
===file===
<?php
                    class LlmAgent {
                        /** @psalm-taint-sink llm_prompt $prompt */
                        public function prompt(string $prompt): string {
                            return "";
                        }
                    }

                    $agent = new LlmAgent();
                    $agent->prompt((string) $_GET["question"]);
===expect===
TaintedLlmPrompt
===ignore===
TODO
