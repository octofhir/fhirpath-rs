import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load the FHIRPath grammar
const fhirpathGrammar = JSON.parse(readFileSync(join(__dirname, 'src/content/grammars/fhirpath.tmGrammar.json'), 'utf8'));

// Create the language object compatible with Shiki v3
const fhirpathLanguage = {
  id: 'fhirpath',
  scopeName: 'source.fhirpath',
  aliases: ['fhirpath', 'fhir'],
  ...fhirpathGrammar
};

export default {
  shiki: {
    langs: [fhirpathLanguage],
  },
};