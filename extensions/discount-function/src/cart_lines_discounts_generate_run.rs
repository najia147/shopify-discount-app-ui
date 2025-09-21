use crate::schema::CartLineTarget; 
use crate::schema::CartLinesDiscountsGenerateRunResult;
use crate::schema::CartOperation;
use crate::schema::DiscountClass;
use crate::schema::OrderDiscountCandidate;
use crate::schema::OrderDiscountCandidateTarget;
use crate::schema::OrderDiscountCandidateValue;
use crate::schema::OrderDiscountSelectionStrategy;
use crate::schema::OrderDiscountsAddOperation;
use crate::schema::OrderSubtotalTarget;
use crate::schema::Percentage;
use crate::schema::ProductDiscountCandidate;
use crate::schema::ProductDiscountCandidateTarget;
use crate::schema::ProductDiscountCandidateValue;
use crate::schema::ProductDiscountSelectionStrategy;
use crate::schema::ProductDiscountsAddOperation;

use super::schema;
use shopify_function::prelude::*;
use shopify_function::Result;

use serde::Deserialize;

#[derive(Deserialize)]
struct DiscountSettings {
    percent: f64,
    excludedTag: String,
}


#[shopify_function]
fn cart_lines_discounts_generate_run(
    input: schema::cart_lines_discounts_generate_run::Input,
) -> Result<schema::CartLinesDiscountsGenerateRunResult> {
    // Only run product discounts if the discount has the PRODUCT class set
    let has_product_discount_class = input
        .discount()
        .discount_classes()
        .contains(&schema::DiscountClass::Product);

    if !has_product_discount_class {
        return Ok(schema::CartLinesDiscountsGenerateRunResult { operations: vec![] });
    }

    // Read settings JSON from metafield
    let settings: DiscountSettings = input
        .discount()
        .metafield()
        .and_then(|mf| Some(mf.value()))
        .and_then(|val| serde_json::from_str::<DiscountSettings>(val).ok())
        .unwrap_or(DiscountSettings {
            percent: 10.0,
            excludedTag: "NO_DISCOUNT".to_string(),
        });

    let mut candidates: Vec<schema::ProductDiscountCandidate> = Vec::new();

   for line in input.cart().lines().iter() {
    match &line.merchandise() {
        schema::cart_lines_discounts_generate_run::input::cart::lines::Merchandise::ProductVariant(variant) => {
            let product = variant.product();

            // ✅ Skip if product has the excluded tag (from metafield settings)
            if product
                .has_tags()
                .iter()
                .any(|ht| ht.tag() == &settings.excludedTag && *ht.has_tag())
            {
                continue;
            }


            let target = schema::ProductDiscountCandidateTarget::CartLine(
                schema::CartLineTarget {
                    id: line.id().clone(),
                    quantity: None,
                },
            );

            let candidate = schema::ProductDiscountCandidate {
                targets: vec![target],
                message: Some(format!("{}% off", settings.percent)),
                value: schema::ProductDiscountCandidateValue::Percentage(
                    schema::Percentage { value: Decimal(settings.percent) },
                ),
                associated_discount_code: None,
            };

            candidates.push(candidate);
        }
        _ => {}
    }
}


    if candidates.is_empty() {
        return Ok(schema::CartLinesDiscountsGenerateRunResult { operations: vec![] });
    }

    // Add a ProductDiscountsAdd operation with selection strategy ALL (apply to all eligible)
    let operations = vec![schema::CartOperation::ProductDiscountsAdd(
        schema::ProductDiscountsAddOperation {
            selection_strategy: schema::ProductDiscountSelectionStrategy::All,
            candidates,
        },
    )];

    Ok(schema::CartLinesDiscountsGenerateRunResult { operations })
}
