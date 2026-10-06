import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EntityAttachmentManagerComponent } from './entity-attachment-manager.component';

describe('EntityAttachmentManagerComponent', () => {
  let component: EntityAttachmentManagerComponent;
  let fixture: ComponentFixture<EntityAttachmentManagerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [EntityAttachmentManagerComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(EntityAttachmentManagerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
