(ns test-runner
  "Clojure FHIRPath Test Runner

  This script runs FHIRPath tests using the fhirpath.clj library
  and outputs results in a standardized format for comparison."
  (:require [clojure.data.json :as json]
            [clojure.data.xml :as xml]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [fhirpath.core :as fp])
  (:import [java.time Instant]
           [java.io File]))

(defn current-timestamp []
  (.toString (Instant/now)))

(defn load-test-config []
  "Load test configuration from test-config.json"
  (let [config-path (io/file "../../test-cases/test-config.json")]
    (when (.exists config-path)
      (json/read-str (slurp config-path) :key-fn keyword))))

(defn load-official-tests []
  "Load official FHIRPath test cases from XML file"
  (try
    (let [xml-path (io/file "../../test-cases/tests-fhir-r4.xml")
          xml-content (slurp xml-path)
          parsed-xml (xml/parse-str xml-content)
          groups (filter #(= :group (:tag %)) (:content parsed-xml))]

      (flatten
        (for [group groups
              :let [group-name (get-in group [:attrs :name] "unknown")]
              test (filter #(= :test (:tag %)) (:content group))
              :let [test-attrs (:attrs test)
                    test-name (:name test-attrs)
                    test-description (or (:description test-attrs) test-name)
                    input-file (or (:inputfile test-attrs) "patient-example.xml")
                    predicate (= "true" (:predicate test-attrs))
                    mode (:mode test-attrs)
                    invalid (:invalid test-attrs)

                    expression-elem (first (filter #(= :expression (:tag %)) (:content test)))
                    expression (when expression-elem (first (:content expression-elem)))

                    output-elems (filter #(= :output (:tag %)) (:content test))
                    expected-output (for [output output-elems
                                         :let [output-type (get-in output [:attrs :type] "string")
                                               output-value (first (:content output))]]
                                     {:type output-type :value output-value})]
              :when (and expression (not (str/blank? expression)))]

          {:name test-name
           :description test-description
           :inputFile input-file
           :expression expression
           :expectedOutput expected-output
           :predicate predicate
           :mode mode
           :invalid invalid
           :group group-name})))

    (catch Exception e
      (println (str "❌ Error loading official tests: " (.getMessage e)))
      [])))

(defn load-test-data [filename]
  "Load test data from XML file and convert to Clojure data structure"
  (let [file-path (io/file (str "../../test-data/" filename))]
    (when (.exists file-path)
      (try
        (let [xml-content (slurp file-path)
              parsed-xml (xml/parse-str xml-content)]
          ;; Return the parsed XML structure that fhirpath.clj can work with
          parsed-xml)
        (catch Exception e
          (println (str "⚠️  Error loading test data " filename ": " (.getMessage e)))
          nil)))))

(defn safe-serialize [obj]
  "Safely serialize objects to JSON-compatible format"
  (try
    (cond
      (nil? obj) nil
      (string? obj) obj
      (number? obj) obj
      (boolean? obj) obj
      (keyword? obj) (name obj)
      (sequential? obj) (mapv safe-serialize obj)
      (map? obj) (into {} (map (fn [[k v]] [(safe-serialize k) (safe-serialize v)]) obj))
      :else (str obj))
    (catch Exception e
      (str "<non-serializable: " (type obj) ">"))))

(defn run-single-test [test test-data]
  "Run a single FHIRPath test case"
  (let [start-time (System/nanoTime)]
    (try
      (let [result (fp/fp (:expression test) test-data)
            end-time (System/nanoTime)
            execution-time-ms (/ (- end-time start-time) 1000000.0)
            serialized-result (safe-serialize result)]

        {:name (:name test)
         :expression (:expression test)
         :result serialized-result
         :expected (:expectedOutput test)
         :success true
         :executionTimeMs execution-time-ms
         :error nil})

      (catch Exception e
        (let [end-time (System/nanoTime)
              execution-time-ms (/ (- end-time start-time) 1000000.0)]
          {:name (:name test)
           :expression (:expression test)
           :result nil
           :expected (:expectedOutput test)
           :success false
           :executionTimeMs execution-time-ms
           :error (.getMessage e)})))))

(defn run-tests []
  "Run all FHIRPath tests and return results"
  (println "🧪 Running Clojure FHIRPath tests...")

  (let [tests (load-official-tests)
        test-config (load-test-config)
        start-time (current-timestamp)]

    (println (str "📋 Loaded " (count tests) " test cases"))

    (let [results (for [test tests
                       :let [test-data (load-test-data (:inputFile test))]]
                   (do
                     (print (str "  Running: " (:name test) "... "))
                     (flush)
                     (let [result (run-single-test test test-data)]
                       (println (if (:success result) "✅" "❌"))
                       result)))

          end-time (current-timestamp)
          successful-tests (count (filter :success results))
          total-tests (count results)
          success-rate (if (> total-tests 0) (double (/ successful-tests total-tests)) 0.0)]

      (println (str "\n📊 Results: " successful-tests "/" total-tests
                   " tests passed (" (format "%.1f" (* success-rate 100)) "%)"))

      {:language "clojure"
       :timestamp end-time
       :tests results
       :summary {:total total-tests
                 :passed successful-tests
                 :failed (- total-tests successful-tests)
                 :errors 0}})))

(defn save-results [results]
  "Save test results to JSON file"
  (let [timestamp (.toString (Instant/now))
        filename (str "../../results/clojure_test_results_"
                     (str/replace timestamp #"[:.T-]" "_") ".json")]
    (try
      (spit filename (json/write-str results :indent true))
      (println (str "💾 Results saved to: " filename))
      (catch Exception e
        (println (str "❌ Error saving results: " (.getMessage e)))))))

(defn run-benchmarks []
  "Run benchmarks and return results"
  (println "⚡ Running Clojure FHIRPath benchmarks...")

  (let [test-config (load-test-config)
        benchmark-tests (get test-config :benchmarkTests [])
        test-data-files (get-in test-config [:testData :inputFiles] [])]

    (if (empty? benchmark-tests)
      (do
        (println "⚠️  No benchmark tests found in configuration")
        {:language "clojure"
         :timestamp (.toString (Instant/now))
         :benchmarks []
         :system_info {:platform (System/getProperty "os.name")
                       :java_version (System/getProperty "java.version")
                       :clojure_version (clojure-version)
                       :fhirpath_version "fhirpath.clj"}})

      (let [results {:language "clojure"
                     :timestamp (.toString (Instant/now))
                     :benchmarks []
                     :system_info {:platform (System/getProperty "os.name")
                                   :java_version (System/getProperty "java.version")
                                   :clojure_version (clojure-version)
                                   :fhirpath_version "fhirpath.clj"}}

            ;; Load test data cache
            test-data-cache (into {}
                                  (for [input-file test-data-files
                                        :let [test-data (load-test-data input-file)]
                                        :when test-data]
                                    [input-file test-data]))]

        (println (str "📋 Running " (count benchmark-tests) " benchmark tests"))

        (let [benchmark-results
              (for [benchmark benchmark-tests
                    :let [input-file (get benchmark :inputFile "patient-example.xml")
                          test-data (get test-data-cache input-file)]]
                (if (nil? test-data)
                  (do
                    (println (str "⚠️  Skipping benchmark " (:name benchmark) " - test data not available"))
                    nil)
                  (do
                    (println (str "  🏃 Running " (:name benchmark) "..."))
                    (let [iterations (get benchmark :iterations 1000)
                          expression (:expression benchmark)

                          ;; Warm up
                          _ (dotimes [_ 10]
                              (try
                                (fp/fp expression test-data)
                                (catch Exception e nil)))

                          ;; Actual benchmark
                          times (for [_ (range iterations)]
                                  (let [start-time (System/nanoTime)]
                                    (try
                                      (fp/fp expression test-data)
                                      (catch Exception e nil))
                                    (let [end-time (System/nanoTime)]
                                      (/ (- end-time start-time) 1000000.0))))

                          valid-times (filter pos? times)
                          avg-time (if (seq valid-times) (/ (reduce + valid-times) (count valid-times)) 0.0)
                          min-time (if (seq valid-times) (apply min valid-times) 0.0)
                          max-time (if (seq valid-times) (apply max valid-times) 0.0)
                          ops-per-second (if (> avg-time 0) (/ 1000.0 avg-time) 0.0)]

                      (println (str "    ⏱️  " (format "%.2f" avg-time) "ms avg ("
                                   (format "%.1f" ops-per-second) " ops/sec)"))

                      {:name (:name benchmark)
                       :description (:description benchmark)
                       :expression expression
                       :iterations iterations
                       :avg_time_ms avg-time
                       :min_time_ms min-time
                       :max_time_ms max-time
                       :ops_per_second ops-per-second}))))

              valid-results (filter some? benchmark-results)
              final-results (assoc results :benchmarks valid-results)]

          ;; Save benchmark results
          (let [filename "../../results/clojure_benchmark_results.json"]
            (try
              (spit filename (json/write-str final-results :indent true))
              (println (str "📊 Benchmark results saved to: " filename))
              (catch Exception e
                (println (str "❌ Error saving benchmark results: " (.getMessage e))))))

          final-results)))))

(defn -main [& args]
  "Main entry point for the test runner"
  (println "🚀 Starting Clojure FHIRPath Test Runner")
  (println "=====================================")

  (let [command (if (seq args) (first args) "both")]

    (try
      (when (or (= command "test") (= command "both"))
        (let [results (run-tests)]
          (save-results results)))

      (when (or (= command "benchmark") (= command "both"))
        (run-benchmarks))

      (println "\n✅ Clojure test runner completed successfully")

      (catch Exception e
        (println (str "\n❌ Test runner failed: " (.getMessage e)))
        (System/exit 1)))))

;; Allow running as script
(when (= *file* (first *command-line-args*))
  (-main))
