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

    let mut candidates: Vec<schema::ProductDiscountCandidate> = Vec::new();

   for line in input.cart().lines().iter() {
    match &line.merchandise() {
        schema::cart_lines_discounts_generate_run::input::cart::lines::Merchandise::ProductVariant(variant) => {
            // Skip if product has the excluded tag
           if *variant.product().has_any_tag() {
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
                message: Some("10% off".to_string()),
                value: schema::ProductDiscountCandidateValue::Percentage(
                    schema::Percentage { value: Decimal(10.0) },
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
