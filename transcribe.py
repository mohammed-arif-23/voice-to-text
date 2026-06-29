import sys
import os

# Suppress Hugging Face download progress bar to keep logs clean
os.environ["HF_HUB_DISABLE_PROGRESS_BARS"] = "1"

# Comprehensive global medical vocabulary prompt.
# This biases Whisper's beam-search token probabilities toward correct clinical spellings.
# Covers: dermatology, antibiotics, antifungals, cardiovascular, allergy, GI, diabetes,
# painkillers, oncology language, dosing instructions, and Indian brand names.
MEDICAL_PROMPT = (
    "Medical clinical dictation transcription. "
    # Dermatology & Skincare
    "Salicylic Acid, Niacinamide, Glycolic Acid, Lactic Acid, Hyaluronic Acid, "
    "Benzoyl Peroxide, Clindamycin, Adapalene, Tretinoin, Retinol, Retinaldehyde, "
    "Ketoconazole, Miconazole, Fluconazole, Clotrimazole, Terbinafine, "
    "Azelaic Acid, Kojic Acid, Arbutin, Tranexamic Acid, Ascorbic Acid, "
    "Coal Tar, Ciclopirox, Clobetasol, Mometasone, Betamethasone, Hydrocortisone, "
    "Calcipotriol, Tacrolimus, Pimecrolimus, Ivermectin, Permethrin, "
    # Antibiotics
    "Amoxicillin, Amoxyclav, Augmentin, Azithromycin, Azithral, Doxycycline, "
    "Cefixime, Cefpodoxime, Cefuroxime, Ceftriaxone, Cephalexin, "
    "Ciprofloxacin, Ciplox, Ofloxacin, Norfloxacin, Levofloxacin, "
    "Metronidazole, Tinidazole, Ornidazole, Nitrofurantoin, Trimethoprim, "
    "Doxycycline, Minocycline, Clarithromycin, Erythromycin, Clindamycin, "
    "Vancomycin, Linezolid, Meropenem, Imipenem, Piperacillin, Tazobactam, "
    # Antifungals & Antivirals
    "Fluconazole, Itraconazole, Voriconazole, Amphotericin, "
    "Acyclovir, Valacyclovir, Oseltamivir, Tenofovir, Lamivudine, "
    # Painkillers & NSAIDs
    "Paracetamol, Dolo, Crocin, Ibuprofen, Aceclofenac, Diclofenac, "
    "Nimesulide, Mefenamic Acid, Naproxen, Etoricoxib, Celecoxib, "
    "Tramadol, Tramacip, Morphine, Codeine, Tapentadol, "
    # GI & Antacids
    "Pantoprazole, Pantocid, Omeprazole, Omez, Rabeprazole, Esomeprazole, "
    "Ranitidine, Famotidine, Domperidone, Ondansetron, Metoclopramide, "
    "Sucralfate, Antacid, Lactulose, Bisacodyl, Mesalamine, "
    # Cardiovascular
    "Amlodipine, Amlokind, Telmisartan, Telma, Losartan, Olmesartan, "
    "Atorvastatin, Rosuvastatin, Simvastatin, Ezetimibe, Fenofibrate, "
    "Metoprolol, Atenolol, Bisoprolol, Carvedilol, Nebivolol, "
    "Ramipril, Enalapril, Lisinopril, Valsartan, Sacubitril, "
    "Aspirin, Clopidogrel, Warfarin, Rivaroxaban, Apixaban, Dabigatran, "
    "Digoxin, Amiodarone, Furosemide, Spironolactone, Hydrochlorothiazide, "
    # Allergy & Respiratory
    "Cetirizine, Levocetirizine, Montelukast, Fexofenadine, Loratadine, "
    "Desloratadine, Ebastine, Bilastine, Rupatadine, Olopatadine, "
    "Salbutamol, Budesonide, Fluticasone, Formoterol, Salmeterol, Tiotropium, "
    "Ipratropium, Theophylline, Doxofylline, Acetylcysteine, Ambroxol, "
    # Diabetes & Endocrinology
    "Metformin, Glycomet, Glimepiride, Gliclazide, Glibenclamide, "
    "Teneligliptin, Sitagliptin, Vildagliptin, Saxagliptin, Alogliptin, "
    "Dapagliflozin, Empagliflozin, Canagliflozin, Pioglitazone, "
    "Insulin Glargine, Insulin Aspart, Insulin Lispro, Insulin NPH, "
    "Levothyroxine, Carbimazole, Methimazole, "
    # Neurology & Psychiatry
    "Gabapentin, Pregabalin, Phenytoin, Carbamazepine, Valproate, Levetiracetam, "
    "Sertraline, Escitalopram, Fluoxetine, Paroxetine, Venlafaxine, "
    "Amitriptyline, Nortriptyline, Clonazepam, Alprazolam, Lorazepam, "
    "Risperidone, Olanzapine, Quetiapine, Haloperidol, "
    "Donepezil, Memantine, "
    # Vitamins & Supplements
    "Vitamin D3, Vitamin B12, Vitamin C, Folic Acid, Iron, Calcium, Zinc, "
    "Omega 3, Multivitamin, Biotin, "
    # Prescribing language
    "tablet capsule syrup drops injection patch cream gel ointment lotion "
    "once daily twice daily three times daily four times daily "
    "after meals before meals at bedtime with water "
    "mg ml mcg IU units "
    "for 5 days for 7 days for 10 days for 14 days for 30 days "
    "continue stop review follow up SOS as needed "
)


def main():
    try:
        from faster_whisper import WhisperModel
        
        # Load the small.en model (highly accurate, fast on CPU)
        model = WhisperModel("small.en", device="cpu", compute_type="int8")
        
        # Signal to the Rust parent process that we are loaded and ready
        print("READY", flush=True)
        
        # Continuous loop reading WAV file paths from stdin
        for line in sys.stdin:
            wav_path = line.strip()
            if not wav_path:
                continue
            if not os.path.exists(wav_path):
                print("", flush=True)
                continue
                
            try:
                # Transcribe with comprehensive medical initial_prompt
                # beam_size=5 for highest accuracy (vs default 1)
                # vad_filter=True to remove background noise / non-speech segments
                segments, _info = model.transcribe(
                    wav_path,
                    beam_size=5,
                    initial_prompt=MEDICAL_PROMPT,
                    language="en",
                    vad_filter=True,
                    vad_parameters=dict(min_silence_duration_ms=400),
                    temperature=0.0,   # Greedy deterministic - no hallucination
                )
                text = " ".join([segment.text for segment in segments]).strip()
                # Print result on a single line and flush
                print(text, flush=True)
            except Exception as inner_e:
                print(f"ERROR: {inner_e}", file=sys.stderr, flush=True)
                print("", flush=True)
                
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr, flush=True)
        sys.exit(1)

if __name__ == "__main__":
    main()
